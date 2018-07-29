// FIXME use_extern_macros
// use serenity::command;

use crate::util::EpsilonEq;

command!(calc_st(_ctx, msg, arg) {
    let st = arg.single::<f64>().unwrap();

    let lift = 10f64.powf(st / 10.0) * 2.0;
    let swing  = (st - 6.0) / 4.0;
    let thrust = (st - 8.0) / 4.0;

    let lift_out =
        if lift < 8.0 {
            lift
        } else {
            lift.round()
        };

    let swing_out =
        if swing < 1.0 {
            format!("1d{}", st - 10.0)
        } else {
            match swing % 1. {
                f if f.eps_eq(0.00) => format!("{}d",   swing.floor()),
                f if f.eps_eq(0.25) => format!("{}d+1", swing.floor()),
                f if f.eps_eq(0.50) => format!("{}d+2", swing.floor()),
                f if f.eps_eq(0.75) => format!("{}d-1", swing.floor() + 1.0),
                _ => unreachable!(),
            }
        };

    let thrust_out =
        if thrust < 1.0 {
            format!("1d{}", st - 12.0)
        } else {
            match thrust % 1f64 {
                f if f.eps_eq(0.00) => format!("{}d",   thrust.floor()),
                f if f.eps_eq(0.25) => format!("{}d+1", thrust.floor()),
                f if f.eps_eq(0.50) => format!("{}d+2", thrust.floor()),
                f if f.eps_eq(0.75) => format!("{}d-1", thrust.floor() + 1.0),
                _ => unreachable!(),
            }
        };

    msg.reply(&format!("**ST** {}: **Basic Lift** {}; **Damage** *Thr* {}, *Sw* {}",
        st, lift_out, thrust_out, swing_out))?;
});
