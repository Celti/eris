use chrono::Utc;
use ddate::DiscordianDate;
use serenity::framework::standard::CreateGroup;

cmd!(Fnord(_ctx, msg), desc: "Receive a message from the Conspiracy.", {
    say!(msg.channel_id, "{}", fnorder::fnorder());
});

cmd!(DDate(_ctx, msg), desc: "PERPETUAL DATE CONVERTER FROM GREGORIAN TO POEE CALENDAR", {
    say!(msg.channel_id, "Today is {}", Utc::today().to_poee());
});

pub fn commands(g: CreateGroup) -> CreateGroup {
    g.cmd("fnord", Fnord::new())
     .cmd("ddate", DDate::new())
}
