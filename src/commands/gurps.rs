fn get_strength(st: f64) -> String {
    let sw_adds;
    let mut sw_dice;
    let sw_frac;
    let thr_adds;
    let mut thr_dice;
    let thr_frac;

    let mut lift: f64 = (st * st) / 5.0;

    if lift > 10.0 {
        lift = lift.floor();
    }

    if st < 11.0 {
        thr_dice = 1.0;
        sw_dice = 1.0;

        match st {
            x if x <= 2.0 => {
                thr_adds = "-6";
                sw_adds = "-5";
            }
            x if x <= 4.0 => {
                thr_adds = "-5";
                sw_adds = "-4";
            }
            x if x <= 6.0 => {
                thr_adds = "-4";
                sw_adds = "-3";
            }
            x if x <= 8.0 => {
                thr_adds = "-3";
                sw_adds = "-2";
            }
            x if x <= 9.0 => {
                thr_adds = "-2";
                sw_adds = "-1";
            }
            x if x <= 10.0 => {
                thr_adds = "-2";
                sw_adds = "";
            }
            _ => unreachable!(),
        }
    } else {
        thr_frac = (st as f64 - 5.0) / 8.0;
        thr_dice = thr_frac.floor();

        match thr_frac % 1.0 {
            x if x < 0.25 => {
                thr_adds = "";
            }
            x if x < 0.50 => {
                thr_adds = "+1";
            }
            x if x < 0.75 => {
                thr_adds = "+2";
            }
            x if x < 1.0 => {
                thr_adds = "-1";
                thr_dice += 1.0;
            }
            _ => unreachable!(),
        };

        sw_frac = (st as f64 - 6.0) / 4.0;
        sw_dice = sw_frac.floor();

        match sw_frac % 1.0 {
            x if x < 0.25 => {
                sw_adds = "";
            }
            x if x < 0.50 => {
                sw_adds = "+1";
            }
            x if x < 0.75 => {
                sw_adds = "+2";
            }
            x if x < 1.0 => {
                sw_adds = "-1";
                sw_dice += 1.0;
            }
            _ => unreachable!(),
        };
    }

    format!(
        "**ST** {}: **Basic Lift** {}; **Damage** *Thr* {}d{}, *Sw* {}d{}",
        st,
        lift,
        thr_dice,
        thr_adds,
        sw_dice,
        sw_adds
    )
}

command!(st(_ctx, msg, arg) {
    let _ = msg.reply(&get_strength(arg.single::<f64>()?));
});
