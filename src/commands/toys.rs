command!(fnord(_ctx, msg) {
    msg.channel_id.say(&::fnorder::fnorder())?;
});

command!(trade(_ctx, msg) {
    use rand::Rng;
    let mut rng = ::rand::thread_rng();

    match rng.gen_range(1, 125) {
         0...25 => msg.channel_id.say("Lizards think your kittens are adorable.")?,
        26...40 => msg.channel_id.say("Griffins hate you for no reason.")?,
        41...70 => msg.channel_id.say("Zebras hate you for no reason.")?,
        71...85 => msg.channel_id.say("Spiders think yiour kittens are adorable.")?,
        _ => {
            msg.channel_id.say(&format!("You've got {} {}",
                rng.gen_range(1.0, 10_000.0),
                rng.choose(&["wood", "catnip", "iron", "minerals", "titanium!", "coal"]).unwrap()
            ))?
        }
    };
});

command!(ddate(_ctx, msg) {
    msg.channel_id.say(&format!("Today is {}",
                                ::ext::ddate::ddate(&::chrono::Utc::now())))?;
});
