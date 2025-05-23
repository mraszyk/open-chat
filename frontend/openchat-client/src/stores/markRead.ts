/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { type Subscriber, type Unsubscriber, writable } from "svelte/store";
import {
    ChatMap,
    MessageContextMap,
    bigIntMax,
    type ChatIdentifier,
    type ChatSummary,
    type MarkReadRequest,
    type MarkReadResponse,
    type Mention,
    type MessageContext,
    type ThreadRead,
    type ThreadSyncDetails,
} from "openchat-shared";
import { unconfirmed } from "./unconfirmed";
import type { OpenChat } from "../openchat";
import { offlineStore } from "./network";
import { ephemeralMessages } from "./ephemeralMessages";

const MARK_READ_INTERVAL = 10 * 1000;

export interface MarkMessagesRead {
    markMessagesRead: (request: MarkReadRequest) => Promise<MarkReadResponse>;
}

export class MessagesRead {
    public readUpTo: number | undefined;
    public threads: Record<number, number>;
    public dateReadPinned: bigint | undefined;

    constructor() {
        this.readUpTo = undefined;
        this.threads = {};
        this.dateReadPinned = undefined;
    }

    get threadsList(): ThreadRead[] {
        return Object.entries(this.threads).map(([threadRootMessageIndex, readUpTo]) => ({
            threadRootMessageIndex: Number(threadRootMessageIndex),
            readUpTo,
        }));
    }

    empty(): boolean {
        return (
            this.readUpTo === undefined &&
            Object.keys(this.threads).length === 0 &&
            this.dateReadPinned === undefined
        );
    }

    markReadUpTo(index: number): void {
        this.readUpTo = Math.max(this.readUpTo ?? 0, index);
    }

    updateThread(rootIndex: number, readUpTo: number): void {
        const current = this.threads[rootIndex];
        if (current === undefined || current < readUpTo) {
            this.threads[rootIndex] = readUpTo;
        }
    }

    setThreads(threads: ThreadRead[]): void {
        this.threads = threads.reduce(
            (rec, t) => {
                rec[t.threadRootMessageIndex] = t.readUpTo;
                return rec;
            },
            {} as Record<number, number>,
        );
    }

    markReadPinned(dateReadPinned: bigint | undefined): void {
        this.dateReadPinned = dateReadPinned;
    }
}

type MessagesReadByChat = ChatMap<MessagesRead>;

export type MessageReadState = {
    serverState: MessagesReadByChat;
    waiting: MessageContextMap<Map<bigint, number>>;
    state: MessagesReadByChat;
};

export class MessageReadTracker {
    private timeout: number | undefined;
    public serverState: MessagesReadByChat = new ChatMap<MessagesRead>();

    /**
     * The waiting structure is either keyed on chatId for normal chat messages or
     * of chatId_threadRootMessageIndex for thread messages
     */
    public waiting: MessageContextMap<Map<bigint, number>> = new MessageContextMap<
        Map<bigint, number>
    >(); // The map is messageId -> (unconfirmed) messageIndex
    public state: MessagesReadByChat = new ChatMap<MessagesRead>();
    private subscribers: Subscriber<MessageReadState>[] = [];
    private executingBatch = false;
    private publishRequired = false;

    public subscribe(sub: Subscriber<MessageReadState>): Unsubscriber {
        this.subscribers.push(sub);
        sub({
            serverState: this.serverState,
            waiting: this.waiting,
            state: this.state,
        });
        return () => {
            this.subscribers = this.subscribers.filter((s) => s !== sub);
        };
    }

    private triggerLoop(api: OpenChat): void {
        this.timeout = window.setTimeout(() => this.sendToServer(api), MARK_READ_INTERVAL);
    }

    start(api: OpenChat): void {
        console.log("starting the mark read poller");
        if (import.meta.env.OC_NODE_ENV !== "test") {
            this.triggerLoop(api);
        }
        if (import.meta.env.OC_NODE_ENV !== "test") {
            window.onbeforeunload = () => this.sendToServer(api);
        }
    }

    stop(): void {
        console.log("stopping the mark read poller");
        if (this.timeout !== undefined) {
            window.clearTimeout(this.timeout);
        }
    }

    private sendToServer(api: OpenChat): void {
        const req = this.state.entries().reduce<MarkReadRequest>((req, [chatId, data]) => {
            if (!data.empty()) {
                req.push({
                    chatId,
                    readUpTo: data.readUpTo,
                    threads: data.threadsList,
                    dateReadPinned: data.dateReadPinned,
                });
            }
            return req;
        }, [] as MarkReadRequest);

        if (req.length > 0) {
            console.log("Sending messages read to the server: ", JSON.stringify(req));
            api.sendMarkReadRequest(req)
                .catch((err) => {
                    console.log("Error calling markMessagesRead", err);
                })
                .finally(() => this.triggerLoop(api));
        } else {
            this.triggerLoop(api);
        }
    }

    /** this will notify all subscribers that something that affects the unread message count has changed */
    public publish(): void {
        if (this.executingBatch) {
            this.publishRequired = true;
            return;
        }
        this.subscribers.forEach((sub) => {
            sub({
                serverState: this.serverState,
                waiting: this.waiting,
                state: this.state,
            });
        });
        this.publishRequired = false;
    }

    private stateForId(chatId: ChatIdentifier): MessagesRead {
        if (!this.state.has(chatId)) {
            this.state.set(chatId, new MessagesRead());
        }
        return this.state.get(chatId)!;
    }

    markMessageRead(
        context: MessageContext,
        messageIndex: number,
        messageId: bigint | undefined,
    ): void {
        const chatState = this.stateForId(context.chatId);
        if (!this.state.has(context.chatId)) {
            this.state.set(context.chatId, new MessagesRead());
        }
        if (messageId !== undefined && unconfirmed.contains(context, messageId)) {
            // if a message is unconfirmed we will just tuck it away until we are told it has been confirmed
            if (!this.waiting.has(context)) {
                this.waiting.set(context, new Map<bigint, number>());
            }
            this.waiting.get(context)?.set(messageId, messageIndex);
        } else if (context.threadRootMessageIndex !== undefined) {
            chatState.updateThread(context.threadRootMessageIndex, messageIndex);
        } else {
            // Mark the chat as read up to the new messageIndex
            chatState.markReadUpTo(messageIndex);
        }
        this.publish();
    }

    markReadUpTo(context: MessageContext, to: number): void {
        const chatState = this.stateForId(context.chatId);
        if (context.threadRootMessageIndex !== undefined) {
            chatState.updateThread(context.threadRootMessageIndex, to);
        } else {
            chatState.markReadUpTo(to);
        }
        this.publish();
    }

    markPinnedMessagesRead(chatId: ChatIdentifier, dateLastPinned: bigint): void {
        this.stateForId(chatId).markReadPinned(dateLastPinned);
        this.publish();
    }

    confirmMessage(context: MessageContext, messageIndex: number, messageId: bigint): boolean {
        // this is called when a message is confirmed so that we can move it from
        // the unconfirmed read to the confirmed read. This means that it will get
        // marked as read on the back end
        if (this.removeUnconfirmedMessage(context, messageId)) {
            this.markMessageRead(context, messageIndex, messageId);
            return true;
        }
        return false;
    }

    removeUnconfirmedMessage(context: MessageContext, messageId: bigint): boolean {
        return this.waiting.get(context)?.delete(messageId) ?? false;
    }

    staleThreadCountForChat(chatId: ChatIdentifier, threads: ThreadSyncDetails[]): number {
        return threads
            .map<number>((thread) => {
                return this.threadReadUpTo(chatId, thread.threadRootMessageIndex) <
                    thread.latestMessageIndex
                    ? 1
                    : 0;
            })
            .reduce((total, n) => total + n, 0);
    }

    threadReadUpTo(chatId: ChatIdentifier, threadRootMessageIndex: number): number {
        const local = this.state.get(chatId)?.threads[threadRootMessageIndex] ?? -1;
        const server = this.serverState.get(chatId)?.threads[threadRootMessageIndex] ?? -1;
        const unconfirmedReadCount =
            this.waiting.get({ chatId, threadRootMessageIndex })?.size ?? 0;

        return Math.max(local + unconfirmedReadCount, server);
    }

    staleThreadsCount(threads: ChatMap<ThreadSyncDetails[]>): number {
        return threads.entries().reduce((total, [chatId, threads]) => {
            const forChat = this.staleThreadCountForChat(chatId, threads);
            return forChat > 0 ? total + forChat : total;
        }, 0);
    }

    unreadThreadMessageCount(
        chatId: ChatIdentifier,
        threadRootMessageIndex: number,
        latestMessageIndex: number,
    ): number {
        return Math.max(
            latestMessageIndex - this.threadReadUpTo(chatId, threadRootMessageIndex),
            0,
        );
    }

    unreadMessageCount(chatId: ChatIdentifier, latestMessageIndex: number | undefined): number {
        // previewed chats will not exist in this.serverState
        if (latestMessageIndex === undefined || !this.serverState.has(chatId)) {
            return 0;
        }

        const readUpToServer = this.serverState.get(chatId)?.readUpTo;
        const readUpToLocal = this.state.get(chatId)?.readUpTo;

        const readUpToConfirmed = Math.max(readUpToServer ?? -1, readUpToLocal ?? -1);
        const readUnconfirmedCount = this.waiting.get({ chatId })?.size ?? 0;

        const total = latestMessageIndex - readUpToConfirmed - readUnconfirmedCount;
        return Math.max(total, 0);
    }

    unreadPinned(chatId: ChatIdentifier, dateLastPinned: bigint | undefined): boolean {
        const readServer = this.serverState.get(chatId)?.dateReadPinned ?? BigInt(0);
        const readLocal = this.state.get(chatId)?.dateReadPinned ?? BigInt(0);
        const dateReadPinned = bigIntMax(readServer, readLocal);
        return (dateLastPinned ?? BigInt(0)) > dateReadPinned;
    }

    getFirstUnreadMessageIndex(
        chatId: ChatIdentifier,
        latestMessageIndex: number | undefined,
    ): number | undefined {
        if (latestMessageIndex === undefined) {
            return undefined;
        }
        const readUpToServer = this.serverState.get(chatId)?.readUpTo;
        const readUpToLocal = this.state.get(chatId)?.readUpTo;

        const readUpToConfirmed = Math.max(readUpToServer ?? -1, readUpToLocal ?? -1);

        if (readUpToConfirmed < latestMessageIndex) {
            const readUnconfirmed = this.waiting.get({ chatId }) ?? new Map();
            const unconfirmedMessageIndexes = [...readUnconfirmed.values()];

            for (let i = readUpToConfirmed + 1; i <= latestMessageIndex; i++) {
                if (!unconfirmedMessageIndexes.includes(i)) {
                    return i;
                }
            }
        }
        return undefined;
    }

    getFirstUnreadMention(chat: ChatSummary): Mention | undefined {
        return chat.membership.mentions.find(
            (m) => !this.isRead({ chatId: chat.id }, m.messageIndex, m.messageId),
        );
    }

    markAllRead(chat: ChatSummary) {
        const latestMessageIndex = chat.latestMessage?.event.messageIndex;
        if (latestMessageIndex !== undefined) {
            if (!this.isRead({ chatId: chat.id }, latestMessageIndex, undefined)) {
                this.markReadUpTo({ chatId: chat.id }, latestMessageIndex);
            }

            if (chat.kind !== "direct_chat") {
                for (const thread of chat.membership.latestThreads) {
                    const context = {
                        chatId: chat.id,
                        threadRootMessageIndex: thread.threadRootMessageIndex,
                    };
                    if (!this.isRead(context, thread.latestMessageIndex, undefined)) {
                        this.markReadUpTo(context, thread.latestMessageIndex);
                    }
                }
            }
        }
    }

    // some operations can be called in a loop. We only want to publish at the end of the
    // loop so that downstream derived stores are not re-computed unnecessarily
    batchUpdate(fn: () => void): void {
        this.executingBatch = true;
        try {
            fn();
        } finally {
            this.executingBatch = false;
            if (this.publishRequired) {
                this.publish();
            }
        }
    }

    syncWithServer(
        chatId: ChatIdentifier,
        readUpTo: number | undefined,
        threads: ThreadRead[],
        dateReadPinned: bigint | undefined,
    ): void {
        const serverState = new MessagesRead();
        serverState.readUpTo = readUpTo;
        serverState.setThreads(threads);
        serverState.markReadPinned(dateReadPinned);
        this.serverState.set(chatId, serverState);

        const state = this.state.get(chatId);
        if (state) {
            if (
                readUpTo !== undefined &&
                state.readUpTo !== undefined &&
                state.readUpTo <= readUpTo
            ) {
                state.readUpTo = undefined;
            }

            // for each thread we get from the server
            // remove the corresponding thread data from the client state unless the client state is more recent
            threads.forEach((t) => {
                const readUpTo = state.threads[t.threadRootMessageIndex];
                if (readUpTo !== undefined && readUpTo <= t.readUpTo) {
                    delete state.threads[t.threadRootMessageIndex];
                }
            });

            if (
                dateReadPinned !== undefined &&
                state.dateReadPinned !== undefined &&
                state.dateReadPinned <= dateReadPinned
            ) {
                state.dateReadPinned = undefined;
            }
        }
        this.publish();
    }

    isRead(context: MessageContext, messageIndex: number, messageId: bigint | undefined): boolean {
        if (messageId !== undefined && unconfirmed.contains(context, messageId)) {
            return this.waiting.get(context)?.has(messageId) ?? false;
        } else if (messageId !== undefined && ephemeralMessages.contains(context, messageId)) {
            return true;
        } else if (context.threadRootMessageIndex !== undefined) {
            const serverState = this.serverState.get(context.chatId);
            if ((serverState?.threads[context.threadRootMessageIndex] ?? -1) >= messageIndex)
                return true;
            const localState = this.state.get(context.chatId);
            if ((localState?.threads[context.threadRootMessageIndex] ?? -1) >= messageIndex)
                return true;
        } else {
            const serverState = this.serverState.get(context.chatId);
            if (serverState?.readUpTo !== undefined && serverState.readUpTo >= messageIndex)
                return true;
            const localState = this.state.get(context.chatId);
            if (localState?.readUpTo !== undefined && localState.readUpTo >= messageIndex)
                return true;
        }
        return false;
    }
}

export const messagesRead = new MessageReadTracker();
export const messageActivityFeedReadUpToLocally = writable<bigint>();

let networkUnsub: Unsubscriber | undefined;

export function startMessagesReadTracker(api: OpenChat): void {
    if (networkUnsub !== undefined) {
        networkUnsub();
    }
    networkUnsub = offlineStore.subscribe((offline) => {
        if (offline) {
            messagesRead.stop();
        } else {
            messagesRead.start(api);
        }
    });
}
