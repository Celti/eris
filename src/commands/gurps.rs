use errors::*;

fn get_strength(st: f64) -> String {
    let sw_adds;
    let mut sw_dice;
    let sw_frac;
    let thr_adds;
    let mut thr_dice;
    let thr_frac;
    let mut lift: f64 = (st*st)/5.0;

    if lift > 10.0 {
        lift = lift.floor();
    }

    if st < 11.0 {
        thr_dice = 1.0;
        sw_dice  = 1.0;
    
        match st {
            0.0       => { thr_adds = "-∞"; sw_adds = "-∞"; }
            1.0 | 2.0 => { thr_adds = "-6"; sw_adds = "-5"; }
            3.0 | 4.0 => { thr_adds = "-5"; sw_adds = "-4"; }
            5.0 | 6.0 => { thr_adds = "-4"; sw_adds = "-3"; }
            7.0 | 8.0 => { thr_adds = "-3"; sw_adds = "-2"; }
            9.0       => { thr_adds = "-2"; sw_adds = "-1"; }
            10.0      => { thr_adds = "-2"; sw_adds = ""; }
            _         => unreachable!()
        }
    } else {
        thr_frac = (st as f64 - 5.0)/8.0;
        thr_dice = thr_frac.floor();
        match thr_frac % 1.0 {
            0.0  | 0.125 => { thr_adds = ""; }
            0.25 | 0.375 => { thr_adds = "+1"; }
            0.50 | 0.625 => { thr_adds = "+2"; }
            0.75 | 0.875 => { thr_adds = "-1"; thr_dice = thr_dice + 1.0; },
            _ => unreachable!()
        };

        sw_frac = (st as f64 - 6.0)/4.0;
        sw_dice = sw_frac.floor();
        match sw_frac % 1.0 {
            0.0  | 0.125 => { sw_adds = ""; }
            0.25 | 0.375 => { sw_adds = "+1"; }
            0.50 | 0.625 => { sw_adds = "+2"; }
            0.75 | 0.875 => { sw_adds = "-1"; sw_dice = sw_dice + 1.0; },
            _ => unreachable!()
        };
    }

    format!("**ST** {}: **Basic Lift** {}; **Damage** *Thr* {}d{}, *Sw* {}d{}", st, lift, thr_dice, thr_adds, sw_dice, sw_adds)
}

command!(st(_ctx, msg, arg) {
    let expr = arg.join("").parse::<f64>().map_err(stringify)?;
    let output = format!("{}: {}", &msg.author.name, get_strength(expr));
    let _ = msg.channel_id.say(&output);
});
