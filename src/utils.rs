use std::fmt::Display;

use serenity::all::{ChannelId, Context, CreateMessage, EditMessage, Mentionable, UserId};

use crate::templates::{Template, TemplateInfo};

pub async fn update_embeds<T: Template>(
    ctx: &Context,
    row: &impl TemplateInfo,
    owner_name: &str,
    thread: impl Into<ChannelId>,
) {
    let thread = thread.into();

    let thread_embed = T::embed(row, owner_name);

    thread
        .edit_message(ctx, thread.get(), EditMessage::new().embed(thread_embed))
        .await
        .unwrap();
}

pub enum Announcement {
    Joined { user: UserId, alternative: bool },
    Left(UserId),
}

impl Announcement {
    pub async fn send(&self, ctx: &Context, channel: ChannelId) {
        channel
            .send_message(ctx, CreateMessage::new().content(format!("{self}")))
            .await
            .unwrap();
    }
}

impl Display for Announcement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Announcement::Joined { user, alternative } if *alternative => {
                write!(f, "{} joined as an alternative", user.mention())
            }
            Announcement::Joined { user, .. } => {
                write!(f, "{} joined the fireteam", user.mention())
            }
            Announcement::Left(user) => write!(f, "{} left the fireteam", user.mention()),
        }
    }
}
