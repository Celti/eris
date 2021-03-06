use serenity::framework::standard::CommandError;
use std::process::Command;
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;

#[command]
#[aliases(super)]
#[description("Calculate the linear value (supervalue) for a given size or speed/range modifier.")]
#[num_args(1)]
fn linear(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let val = args.single::<f64>()?;
    let cof = ((val % 6.0) + 2.0) / 6.0;
    let mag = (val / 6.0).trunc();
    let raw = 10f64.powf(cof) * 10f64.powf(mag);

    let rnd = {
        if (val + 1.0) % 6.0 == 0.0 {
            10f64.powf((raw.log10() - 1.0).floor())
        } else {
            10f64.powf(raw.log10().floor())
        }
    };

    reply!(ctx, msg, "Size: {}; Linear Value: {}", val, (raw / rnd).round() * rnd);

    Ok(())
}

#[command]
#[aliases(speed, sr)]
#[description("Calculate the speed/range penalty for a given linear measurement.")]
#[min_args(1)]
fn range(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    reply!(ctx, msg, "{}", -sm(yards(args.message())?));

    Ok(())
}

#[command]
#[aliases(sm)]
#[description("Calculate the size modifier for a given linear measurement.")]
#[min_args(1)]
fn size(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    reply!(ctx, msg, "{}", sm(yards(args.message())?));

    Ok(())
}

#[command]
#[description("Calculate Basic Lift and damage for a given ST (using KYOS).")]
#[num_args(1)]
fn st(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let st = args.single::<f64>()?;

    let lift = {
        let lift = 10f64.powf(st / 10.0) * 2.0;
        let ord  = 10f64.powf((lift.log10() - 1.0).floor());
        (lift / ord).round() * ord
    };

    let swing = {
        let swing = (st - 6.0) / 4.0;
        if swing < 1.0 {
            format!("*Sw* 1d{}", st - 10.0)
        } else {
            #[allow(clippy::float_cmp)] // Matches exactly within desired range.
            match swing % 1f64 {
                f if f == 0.00 => format!("*Sw* {}d",   swing.floor()),
                f if f == 0.25 => format!("*Sw* {}d+1", swing.floor()),
                f if f == 0.50 => format!("*Sw* {}d+2", swing.floor()),
                f if f == 0.75 => format!("*Sw* {}d-1", swing.floor() + 1.0),
                _ => unreachable!(),
            }
        }
    };

    let thrust = {
        let thrust = (st - 8.0) / 4.0;
        if thrust < 1.0 {
            format!("*Thr* 1d{}", st - 12.0)
        } else {
            #[allow(clippy::float_cmp)] // Matches exactly within desired range.
            match thrust % 1f64 {
                f if f == 0.00 => format!("*Thr* {}d, ",   thrust.floor()),
                f if f == 0.25 => format!("*Thr* {}d+1, ", thrust.floor()),
                f if f == 0.50 => format!("*Thr* {}d+2, ", thrust.floor()),
                f if f == 0.75 => format!("*Thr* {}d-1, ", thrust.floor() + 1.0),
                _ => unreachable!(),
            }
        }
    };

    reply!(ctx, msg, "**ST** {}: **Basic Lift** {}; **Damage** {}{}", st, lift, thrust, swing);

    Ok(())
}

group!({
    name: "GURPS",
    options: {},
    commands: [linear, range, size, st]
});

fn sm(yards: f64) -> f64 {
    let ord = 10f64.powf(yards.log10().floor());
    let mul = ord.log10() * 6.0;
    let val = yards / ord;

    mul + match val {
        f if f > 7.0 => 4f64,
        f if f > 5.0 => 3f64,
        f if f > 3.0 => 2f64,
        f if f > 2.0 => 1f64,
        f if f > 1.5 => 0f64,
        f if f > 1.0 => -1f64,
        f if f > 0.0 => -2f64,
        _ => std::f64::NAN,
    }
}

fn yards(dist: &str) -> Result<f64, CommandError> {
    let output = Command::new("/usr/bin/units")
        .arg("--terse")
        .arg("--")
        .arg(dist)
        .arg("yards")
        .env("UNITS_ENGLISH", "US")
        .output()?;

    let yards: f64 = if output.status.success() {
        String::from_utf8(output.stdout)?.trim().parse()?
    } else {
        dist.trim().parse()?
    };

    Ok(yards)
}
