use chrono::Utc;
use ddate::DiscordianDate;

cmd!(Fnord(_ctx, msg), aliases: ["fnord"], desc: "Receive a message from the Conspiracy.", {
    say!(msg.channel_id, "{}", fnorder::fnorder());
});

cmd!(DDate(_ctx, msg), aliases: ["ddate"], desc: "PERPETUAL DATE CONVERTER FROM GREGORIAN TO POEE CALENDAR", {
    say!(msg.channel_id, "Today is {}", Utc::today().to_poee());
});

grp![Fnord, DDate];
