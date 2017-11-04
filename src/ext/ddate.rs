use chrono::Datelike;

const APOSTLES: [&str; 5] = [
    "Mungday",
    "Mojoday",
    "Syaday",
    "Zaraday",
    "Maladay"
];
const HOLYDAYS: [&str; 5] = [
    "Chaoflux",
    "Discoflux",
    "Confuflux",
    "Bureflux",
    "Afflux"
];
const SEASONS: [&str; 5] = [
    "Chaos",
    "Discord",
    "Confusion",
    "Bureaucracy",
    "The Aftermath",
];
const WEEKDAYS: [&str; 5] = [
    "Sweetmorn",
    "Boomtime",
    "Pungenday",
    "Prickle-Prickle",
    "Setting Orange",
];

const APOSTLE_HOLYDAY: usize = 5;
const LEAP_DAY: usize = 59;
const SEASON_DAYS: usize = 73;
const SEASON_HOLYDAY: usize = 50;
const WEEK_DAYS: usize = 5;
const YEAR_OFFSET: i32 = 1166;

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && year % 100 != 0 || year % 400 == 0
}

fn ordinalize(num: usize) -> String {
    let s = format!("{}", num);

    let suffix = if s.ends_with('1') && !s.ends_with("11") {
        "st"
    } else if s.ends_with('2') && !s.ends_with("12") {
        "nd"
    } else if s.ends_with('3') && !s.ends_with("13") {
        "rd"
    } else {
        "th"
    };

    format!("{}{}", s, suffix)
}

pub fn ddate<T: Datelike>(date: &T) -> String {
    let day = date.ordinal0() as usize;
    let leap = is_leap_year(date.year());
    let year = date.year() + YEAR_OFFSET;

    if leap && day == LEAP_DAY {
        return format!("St. Tib's Day, in the YOLD {}", year);
    }

    let day_offset = if leap && day > LEAP_DAY { day - 1 } else { day };

    let day_of_season = day_offset % SEASON_DAYS + 1;

    let season = SEASONS[day_offset / SEASON_DAYS];
    let weekday = WEEKDAYS[day_offset % WEEK_DAYS];

    let holiday = if day_of_season == APOSTLE_HOLYDAY {
        format!("\nCelebrate {}", APOSTLES[day_offset / SEASON_DAYS])
    } else if day_of_season == SEASON_HOLYDAY {
        format!("\nCelebrate {}", HOLYDAYS[day_offset / SEASON_DAYS])
    } else {
        String::with_capacity(0)
    };

    format!(
        "{}, the {} day of {} in the YOLD {}{}",
        weekday,
        ordinalize(day_of_season),
        season,
        year,
        holiday
    )
}
