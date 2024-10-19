use std::collections::HashMap;

use chrono_tz::{America, Asia, Europe, Tz};
use lazy_static::lazy_static;
use serenity::{
    all::{Context, UserId},
    prelude::TypeMapKey,
};

lazy_static! {
    static ref LOCALE_TO_TIMEZONE: HashMap<&'static str, chrono_tz::Tz> = {
        let mut m = HashMap::new();
        m.insert("id", Asia::Jakarta);
        m.insert("da", Europe::Copenhagen);
        m.insert("de", Europe::Berlin);
        m.insert("en-GB", Europe::London);
        // m.insert("en-US", chrono_tz::America::New_York);
        m.insert("es-ES", Europe::Madrid);
        m.insert("es-419", America::Mexico_City);
        m.insert("fr", Europe::Paris);
        m.insert("hr", Europe::Zagreb);
        m.insert("it", Europe::Rome);
        m.insert("lt", Europe::Vilnius);
        m.insert("hu", Europe::Budapest);
        m.insert("nl", Europe::Amsterdam);
        m.insert("no", Europe::Oslo);
        m.insert("pl", Europe::Warsaw);
        m.insert("pt-BR", America::Sao_Paulo);
        m.insert("ro", Europe::Bucharest);
        m.insert("fi", Europe::Helsinki);
        m.insert("sv-SE", Europe::Stockholm);
        m.insert("vi", Asia::Ho_Chi_Minh);
        m.insert("tr", Europe::Istanbul);
        m.insert("cs", Europe::Prague);
        m.insert("el", Europe::Athens);
        m.insert("bg", Europe::Sofia);
        m.insert("ru", Europe::Moscow);
        m.insert("uk", Europe::Kiev);
        m.insert("hi", Asia::Kolkata);
        m.insert("th", Asia::Bangkok);
        m.insert("zh-CN", Asia::Shanghai);
        m.insert("ja", Asia::Tokyo);
        m.insert("zh-TW", Asia::Taipei);
        m.insert("ko", Asia::Seoul);
        m
    };

}

pub struct TimezoneManager;

impl TimezoneManager {
    pub async fn foo<'a>(ctx: &'a Context, user_id: impl Into<UserId>, local: &'a str) -> &'a Tz {
        let timezone = LOCALE_TO_TIMEZONE.get(local).unwrap_or(&chrono_tz::UTC);

        {
            let mut data = ctx.data.write().await;
            let manager = data
                .get_mut::<Self>()
                .expect("TimeZoneManager is in the context");
            manager.insert(user_id.into(), *timezone);
        }

        timezone
    }
}

impl TypeMapKey for TimezoneManager {
    type Value = HashMap<UserId, Tz>;
}
