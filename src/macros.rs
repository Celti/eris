#[macro_export]
macro_rules! err_log {
    ($e:expr) => {
        if let Err(err) = $e {
            log::warn!("[{}:{}] {:?}", line!(), column!(), err);
        }
    };
}

#[macro_export]
macro_rules! say {
    ($ctx:expr, $msg:expr, $($arg:tt)*) => { $crate::err_log!($msg.channel_id.say(&$ctx.http, &std::format!($($arg)*))) }
}

#[macro_export]
macro_rules! reply {
    ($ctx:expr, $msg:expr, $($arg:tt)*) => { $crate::err_log!($msg.reply(&$ctx, &std::format!($($arg)*))) }
}
