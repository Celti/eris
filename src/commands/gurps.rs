command!(calc_st(_ctx, msg, arg) {
    let st = arg.single::<f64>().unwrap();

    let lift = 10f64.powf(st / 10.0).floor() * 2.0;
    let swing  = (st - 6.0) / 4.0;
    let thrust = (st - 8.0) / 4.0;

    let swing_out = 
        if swing < 1.0 {
            format!("1d{}", st - 10.0)
        } else {
            match swing % 1. {
                f if f == 0.00 => format!("{}d",   swing.floor()),
                f if f == 0.25 => format!("{}d+1", swing.floor()),
                f if f == 0.50 => format!("{}d+2", swing.floor()),
                f if f == 0.75 => format!("{}d-1", swing.floor() + 1.0),
                _ => unreachable!(),
            }
        };

    let thrust_out = 
        if thrust < 1.0 {
            format!("1d{}", st - 12.0)
        } else {
            match thrust % 1f64 {
                f if f == 0.00 => format!("{}d",   thrust.floor()),
                f if f == 0.25 => format!("{}d+1", thrust.floor()),
                f if f == 0.50 => format!("{}d+2", thrust.floor()),
                f if f == 0.75 => format!("{}d-1", thrust.floor() + 1.0),
                _ => unreachable!(),
            }
        };

    msg.reply(&format!("**ST** {}: **Basic Lift** {}; **Damage** *Thr* {}, *Sw* {}",
        st, lift, thrust_out, swing_out))?;
});

command!(reaction(_ctx, msg, arg) {
    let _reactions = btreemap! {
        "general" => btreemap! {
            "excellent" => "The NPC is extremely impressed by the PCs, and acts in their best interests at all times, within the limits of their own abilities.",
            "very_good" => "The NPC thinks highly of the PCs and is quite helpful and friendly.",
            "good" => "The NPC likes the PCs and is helpful within reasonable, everyday limits.",
            "neutral" => "The NPC ignores the PCs as much as possible.",
            "poor" => "The NPC is unimpressed. They may act against the PCs if there is much profit and/or little danger in it.",
            "bad" => "The NPC cares nothing for the PCs and acts against them if they profit by doing so.",
            "very_bad" => "The NPC dislikes the PCs and acts against them if convenient.",
            "disastrous" => "The NPC hates the PCs and acts in their worst interest.",
        },
        "combat" => btreemap! {
            "excellent" => "The NPCs are extremely friendly, and may even join the party temporarily.\nThe PCs may request aid or information or give information; roll again at +5.\nIf a fight is in progress, the NPCs surrender.",
            "very_good" => "The NPCs are friendly. Even sworn foes find an excuse to let the PCs go… for now.\nThe PCs may request aid or information or give information; roll again at +3.\nIf a fight is in progress, the NPCs flee if they can, or surrender.",
            "good" => "The NPCs find the PCs likable, or too formidable to attack.\nThe PCs may request aid or information or give information; roll again at +1.\nIf a fight is in progress, the NPCs flee if they can.",
            "neutral" => "The NPCs go their own way, and let the PCs go theirs.\nIf a fight is in progress, the NPCs try to back off.",
            "poor" => "The NPCs shout threats or insults and demand the PCs leave the area. If the PCs stay, the NPCs attack unless outnumbered; if outnumbered they flee.\nA fight in progress continues.",
            "bad" => "The NPCs attack unless outnumbered; if outnumbered they flee, possibly attempting an ambush later.\nA fight in progress continues.",
            "very_bad" => "The NPCs attack, and flee only if they see they have no chance.\nA fight in progress continues.",
            "disastrous" => "The NPCs attack viciously, asking no quarter and giving none.",
        },
        "confrontation" => btreemap! {
            "excellent" => "The PCs are treated deferentially and offered assistance.",
            "very_good" => "The PCs are accepted as legitimate and make a good impression: +2 on further reaction rolls.",
            "good" => "The PCs are accepted as legitimate.",
            "neutral" => "The PCs are questioned for a few minutes and then allowed to go about their business. The questioners will have consciously noticed them and may remember them.",
            "poor" => "The PCs are detained and questioned for an hour. If they are uncooperative, make a “potential combat” roll; on a Bad or worse result they are physically mistreated or forcibly subdued.",
            "bad" => "The PCs are detained and questioned for at least a few hours. If they are uncooperative, make a “potential combat” roll at -2; on a Bad or worse result they are physically mistreated or forcibly subdued.",
            "very_bad" => "The PCs are arrested and charged with a crime. If they are uncooperative, make a “potential combat” roll at -2; on a Bad or worse result they are physically mistreated or forcibly subdued.",
            "disastrous" => "The PCs are arrested and charged with a crime. In the course of being arrested, they are physically mistreated or forcibly subdued.",
        },
        "admission" => btreemap! {
            "excellent" => "Request for entry granted enthusastically; +2 on subsequent reaction rolls during this visit.",
            "very_good" => "Request for entry granted immediately.",
            "good" => "Request for entry granted with mild restrictions, such as leaving weapons at the door.",
            "neutral" => "Request for entry granted after a delay to get approval from someone in authority.",
            "poor" => "Request for entry denied, but bribes, pleas, or threats might work. The PCs may roll again at -2.",
            "bad" => "No chance of getting in; further attempts will be ignored.",
            "very_bad" => "No chance of getting in; further attempts will provoke a “potential combat” roll at -2.",
            "disastrous" => "No chance of getting in; make an immediate “potential combat” roll at -2.",
        },
        "commercial" => btreemap! {
            "excellent" => "When selling: The merchant asks the fair price, and accepts any offer of at least 50% of the fair price.\nWhen buying: The merchant offers the fair price, and agrees to pay up to 200% of the fair price.\nFor an offer outside of these limits, the merchant proposes the limit prices. They also offer help and advice.",
            "very_good" => "When selling: The merchant asks the fair price, and accepts any offer of at least 80% of the fair price.\nWhen buying: The merchant offers the fair price, and agrees to pay up to 150% of the fair price.\nThe merchant also offers help and advice.",
            "good" => "The merchant buys and sells at fair prices, and volunteers useful information or small bits of help if possible.",
            "neutral" => "The merchant buys and sells at fair prices.",
            "poor" => "When selling: The merchant asks for 120% of the fair price, and accepts the fair price.\nWhen buying: The merchant offers 75% of the fair price, and agrees to pay the fair price.",
            "bad" => "When selling: The merchant asks twice the fair price, and accepts the fair price.\nWhen buying: The merchant offers half the fair price, and agrees to pay the fair price.",
            "very_bad" => "When selling: The merchant asks three times the fair price, and accepts 150% the fair price.\nWhen buying: The merchant offers 1/3 the fair price, and agrees to pay 2/3 the fair price.",
            "disastrous" => "The merchant wants nothing to do with the PCs. Make an immediate “potential combat” roll at -2.",
        },
        "hiring" => btreemap! {
            "excellent" => "The organisation values the PC highly and will design a new job around their qualifications, if necessary. If they take the job, they gain higher salary and Rank (if applicable), and can spend earned points to acquire the organisation as a Patron.",
            "very_good" => "The organisation will hire the PC at their established salary range and equivalent Rank (if applicable).",
            "good" => "The organisation will hire the PC for an entry-level job in their field.",
            "neutral" => "The organisation will hire the PC for unskilled labour, but not for a skilled job.",
            "poor" => "The organisation does not hire the PC.",
            "bad" => "The organisation finds the PC's qualifications unsuitable; future applications are at a cumulative -2.",
            "very_bad" => "The organisation rejects the PC and is not open to future inquiries. If they persist or return, make a “confrontation with authority” roll at -2.",
            "disastrous" => "The organisation is hostile to the PC and will act against them if it can: calling the police, reporting them to the authorities, blacklisting them, etc. This often leads to a “confrontation with authority” roll at -2.",
        },
        "aid" => btreemap! {
            "excellent" => "Requests for aid are granted, and extra aid is offered; the NPCs do everything they can to help.",
            "very_good" => "Requests for aid are granted unless they are totally unreasonable. The NPCs volunteer any relevant information they have freely.",
            "good" => "Reasonable requests for aid are granted. Complex requests are denied, but the PCs can try again at -2.",
            "neutral" => "Simple requests for aid are granted. Complex requests are denied, but the PCs can try again at -2.",
            "poor" => "The request for aid is denied, but the PCs can try again at a cumulative -2. Bribes, threats, and pleas may work.",
            "bad" => "The request for aid is denied. The NPC goes about their business, ignoring the PCs.",
            "very_bad" => "The request for aid is denied. Make a “potential combat” roll; no reaction better than Neutral is possible. If combat is called for but not possible, the NPC opposes the PCs in some other fashion.",
            "disastrous" => "The request for aid is denied totally. Make a “potential combat” roll at -4; no reaction better than Neutral is possible. If combat is called for but not possible, the NPC opposes the PCs in any way possible.",
        },
        "information" => btreemap! {
            "excellent" => "",
            "very_good" => "",
            "good" => "",
            "neutral" => "",
            "poor" => "",
            "bad" => "",
            "very_bad" => "",
            "disastrous" => "",
        },
        "response" => btreemap! {
            "excellent" => "",
            "very_good" => "",
            "good" => "",
            "neutral" => "",
            "poor" => "",
            "bad" => "",
            "very_bad" => "",
            "disastrous" => "",
        },
        "recreation" => btreemap! {
            "excellent" => "",
            "very_good" => "",
            "good" => "",
            "neutral" => "",
            "poor" => "",
            "bad" => "",
            "very_bad" => "",
            "disastrous" => "",
        },
        "seduction" => btreemap! {
            "excellent" => "",
            "very_good" => "",
            "good" => "",
            "neutral" => "",
            "poor" => "",
            "bad" => "",
            "very_bad" => "",
            "disastrous" => "",
        },
        "loyalty" => btreemap! {
            "excellent" => "",
            "very_good" => "",
            "good" => "",
            "neutral" => "",
            "poor" => "",
            "bad" => "",
            "very_bad" => "",
            "disastrous" => "",
        },
    };

    use rand::{self, distributions::{IndependentSample, Range}};

    let mut rng = rand::thread_rng();
    let roll: i64 = (0..3).map(|_| Range::new(1,7).ind_sample(&mut rng)).sum();

    let modifier = arg.single::<i64>()?;

    match roll + modifier {
        x if x > 18 => { /* Excellent */
            msg.reply("You got an Excellent reaction.\nThe NPC is extremely impressed by the PCs, and acts in their best interests at all times, within the limits of their own abilities.")?;
        }
        x if x > 15 => { /* Very Good */
            msg.reply("You got a Very Good reaction.\nThe NPC thinks highly of the PCs and is quite helpful and friendly.")?;
        }
        x if x > 12 => { /* Good */ 
            msg.reply("You got a Good reaction.\nThe NPC likes the PCs and is helpful within reasonable, everyday limits.")?;
         }
        x if x > 9 => { /* Neutral */
            msg.reply("You got a Neutral reaction.\nThe NPC ignores the PCs as much as possible.")?;
        }
        x if x > 6 => { /* Poor */
            msg.reply("You got a Poor reaction.\nThe NPC is unimpressed. He may act against the PCs if there is much profit in it, or little danger.")?;
        }
        x if x > 3 => { /* Bad */
            msg.reply("You got a Bad reaction.\nThe NPC cares nothing for the PCs and acts against them if he can profit by doing so.")?;
        }
        x if x > 0 => { /* Very Bad */
            msg.reply("You got a Very Bad reaction.\nThe NPC dislikes the PCs and acts against them if it's convenient.")?;
        }
        _ => { /* Disastrous */
            msg.reply("You got a Disastrous reaction.\nThe NPC hates the PCs and acts in their worst interests.")?;
        }
    }
});
