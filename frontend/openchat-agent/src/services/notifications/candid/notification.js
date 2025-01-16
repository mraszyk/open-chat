import { IDL } from "@dfinity/candid"

export const Notification = IDL.Variant({
  'GroupReactionAdded' : IDL.Record({
    'added_by_name' : IDL.Text,
    'group_avatar_id' : IDL.Opt(IDL.Nat),
    'message_event_index' : IDL.Nat32,
    'added_by' : IDL.Principal,
    'added_by_display_name' : IDL.Opt(IDL.Text),
    'chat_id' : IDL.Principal,
    'thread_root_message_index' : IDL.Opt(IDL.Nat32),
    'group_name' : IDL.Text,
    'reaction' : IDL.Text,
    'message_index' : IDL.Nat32,
  }),
  'ChannelMessageTipped' : IDL.Record({
    'tip' : IDL.Text,
    'channel_id' : IDL.Nat32,
    'tipped_by_display_name' : IDL.Opt(IDL.Text),
    'community_id' : IDL.Principal,
    'message_event_index' : IDL.Nat32,
    'channel_name' : IDL.Text,
    'tipped_by' : IDL.Principal,
    'community_avatar_id' : IDL.Opt(IDL.Nat),
    'community_name' : IDL.Text,
    'tipped_by_name' : IDL.Text,
    'thread_root_message_index' : IDL.Opt(IDL.Nat32),
    'channel_avatar_id' : IDL.Opt(IDL.Nat),
    'message_index' : IDL.Nat32,
  }),
  'DirectMessageTipped' : IDL.Record({
    'tip' : IDL.Text,
    'username' : IDL.Text,
    'message_event_index' : IDL.Nat32,
    'them' : IDL.Principal,
    'display_name' : IDL.Opt(IDL.Text),
    'user_avatar_id' : IDL.Opt(IDL.Nat),
    'thread_root_message_index' : IDL.Opt(IDL.Nat32),
    'message_index' : IDL.Nat32,
  }),
  'DirectMessage' : IDL.Record({
    'image_url' : IDL.Opt(IDL.Text),
    'sender_display_name' : IDL.Opt(IDL.Text),
    'sender_avatar_id' : IDL.Opt(IDL.Nat),
    'sender' : IDL.Principal,
    'sender_name' : IDL.Text,
    'message_text' : IDL.Opt(IDL.Text),
    'message_type' : IDL.Text,
    'event_index' : IDL.Nat32,
    'thread_root_message_index' : IDL.Opt(IDL.Nat32),
    'crypto_transfer' : IDL.Opt(
      IDL.Record({
        'recipient' : IDL.Principal,
        'ledger' : IDL.Principal,
        'recipient_username' : IDL.Opt(IDL.Text),
        'amount' : IDL.Nat,
        'symbol' : IDL.Text,
      })
    ),
    'message_index' : IDL.Nat32,
  }),
  'ChannelReactionAdded' : IDL.Record({
    'channel_id' : IDL.Nat32,
    'community_id' : IDL.Principal,
    'added_by_name' : IDL.Text,
    'message_event_index' : IDL.Nat32,
    'added_by' : IDL.Principal,
    'channel_name' : IDL.Text,
    'community_avatar_id' : IDL.Opt(IDL.Nat),
    'added_by_display_name' : IDL.Opt(IDL.Text),
    'community_name' : IDL.Text,
    'thread_root_message_index' : IDL.Opt(IDL.Nat32),
    'channel_avatar_id' : IDL.Opt(IDL.Nat),
    'reaction' : IDL.Text,
    'message_index' : IDL.Nat32,
  }),
  'DirectReactionAdded' : IDL.Record({
    'username' : IDL.Text,
    'message_event_index' : IDL.Nat32,
    'them' : IDL.Principal,
    'display_name' : IDL.Opt(IDL.Text),
    'user_avatar_id' : IDL.Opt(IDL.Nat),
    'thread_root_message_index' : IDL.Opt(IDL.Nat32),
    'reaction' : IDL.Text,
    'message_index' : IDL.Nat32,
  }),
  'GroupMessage' : IDL.Record({
    'image_url' : IDL.Opt(IDL.Text),
    'group_avatar_id' : IDL.Opt(IDL.Nat),
    'sender_display_name' : IDL.Opt(IDL.Text),
    'sender' : IDL.Principal,
    'sender_name' : IDL.Text,
    'message_text' : IDL.Opt(IDL.Text),
    'message_type' : IDL.Text,
    'chat_id' : IDL.Principal,
    'event_index' : IDL.Nat32,
    'thread_root_message_index' : IDL.Opt(IDL.Nat32),
    'group_name' : IDL.Text,
    'crypto_transfer' : IDL.Opt(
      IDL.Record({
        'recipient' : IDL.Principal,
        'ledger' : IDL.Principal,
        'recipient_username' : IDL.Opt(IDL.Text),
        'amount' : IDL.Nat,
        'symbol' : IDL.Text,
      })
    ),
    'message_index' : IDL.Nat32,
  }),
  'GroupMessageTipped' : IDL.Record({
    'tip' : IDL.Text,
    'tipped_by_display_name' : IDL.Opt(IDL.Text),
    'group_avatar_id' : IDL.Opt(IDL.Nat),
    'message_event_index' : IDL.Nat32,
    'tipped_by' : IDL.Principal,
    'tipped_by_name' : IDL.Text,
    'chat_id' : IDL.Principal,
    'thread_root_message_index' : IDL.Opt(IDL.Nat32),
    'group_name' : IDL.Text,
    'message_index' : IDL.Nat32,
  }),
  'AddedToChannel' : IDL.Record({
    'channel_id' : IDL.Nat32,
    'community_id' : IDL.Principal,
    'added_by_name' : IDL.Text,
    'added_by' : IDL.Principal,
    'channel_name' : IDL.Text,
    'community_avatar_id' : IDL.Opt(IDL.Nat),
    'added_by_display_name' : IDL.Opt(IDL.Text),
    'community_name' : IDL.Text,
    'channel_avatar_id' : IDL.Opt(IDL.Nat),
  }),
  'ChannelMessage' : IDL.Record({
    'channel_id' : IDL.Nat32,
    'community_id' : IDL.Principal,
    'image_url' : IDL.Opt(IDL.Text),
    'sender_display_name' : IDL.Opt(IDL.Text),
    'sender' : IDL.Principal,
    'channel_name' : IDL.Text,
    'community_avatar_id' : IDL.Opt(IDL.Nat),
    'community_name' : IDL.Text,
    'sender_name' : IDL.Text,
    'message_text' : IDL.Opt(IDL.Text),
    'message_type' : IDL.Text,
    'event_index' : IDL.Nat32,
    'thread_root_message_index' : IDL.Opt(IDL.Nat32),
    'channel_avatar_id' : IDL.Opt(IDL.Nat),
    'crypto_transfer' : IDL.Opt(
      IDL.Record({
        'recipient' : IDL.Principal,
        'ledger' : IDL.Principal,
        'recipient_username' : IDL.Opt(IDL.Text),
        'amount' : IDL.Nat,
        'symbol' : IDL.Text,
      })
    ),
    'message_index' : IDL.Nat32,
  }),
});

