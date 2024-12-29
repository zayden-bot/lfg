use std::collections::HashSet;

use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
    EditThread, ForumTagId,
};

use crate::Result;

pub struct TagsComponent;

impl TagsComponent {
    pub async fn run(ctx: &Context, interaction: &ComponentInteraction) -> Result<()> {
        let tag_ids = match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => values
                .iter()
                .map(|x| x.parse::<u64>().unwrap())
                .collect::<Vec<_>>(),
            _ => unreachable!("Expected string select"),
        };

        let mut channel = interaction
            .channel_id
            .to_channel(ctx)
            .await
            .unwrap()
            .guild()
            .unwrap();

        let mut new_tags = channel
            .applied_tags
            .iter()
            .map(|x| x.get())
            .collect::<HashSet<_>>();

        for tag_id in tag_ids {
            if new_tags.insert(tag_id) {
                new_tags.remove(&tag_id);
            }
        }

        channel
            .edit_thread(
                ctx,
                EditThread::new().applied_tags(new_tags.into_iter().map(|id| ForumTagId::new(id))),
            )
            .await
            .unwrap();

        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await
            .unwrap();

        Ok(())
    }
}
