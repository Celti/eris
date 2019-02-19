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
    ($msg:expr, $($arg:tt)*) => { $crate::err_log!($msg.channel_id.say(&std::format!($($arg)*))) }
}

#[macro_export]
macro_rules! reply {
    ($msg:expr, $($arg:tt)*) => { $crate::err_log!($msg.reply(&std::format!($($arg)*))) }
}

#[macro_export]
macro_rules! __cmd_opt {
    (@munch () -> {$(($id:ident: $ex:expr))*}) => {
        ::std::sync::Arc::new(
            ::serenity::framework::standard::CommandOptions {
                $($id: $ex,)*
                ..::std::default::Default::default()
            }
        )
    };

    (@munch ((bucket: $ex:expr) $($next:tt)*) -> {$($output:tt)*}) => {
        __cmd_opt!(@munch ($($next)*) -> { $($output)* (bucket: Some($ex.to_string())) } );
    };

    (@munch ((desc: $ex:expr) $($next:tt)*) -> {$($output:tt)*}) => {
        __cmd_opt!(@munch ($($next)*) -> { $($output)* (desc: Some($ex.to_string())) } );
    };

    (@munch ((example: $ex:expr) $($next:tt)*) -> {$($output:tt)*}) => {
        __cmd_opt!(@munch ($($next)*) -> { $($output)* (example: Some($ex.to_string())) } );
    };

    (@munch ((usage: $ex:expr) $($next:tt)*) -> {$($output:tt)*}) => {
        __cmd_opt!(@munch ($($next)*) -> { $($output)* (usage: Some($ex.to_string())) } );
    };

    (@munch ((aliases: $ex:expr) $($next:tt)*) -> {$($output:tt)*}) => {
        __cmd_opt!(@munch ($($next)*) -> { $($output)* (aliases: $ex.iter().map(|s| s.to_string()).collect()) } );
    };

    (@munch ((allowed_roles: $ex:expr) $($next:tt)*) -> {$($output:tt)*}) => {
        __cmd_opt!(@munch ($($next)*) -> { $($output)* (allowed_roles: $ex.iter().map(|s| s.to_string()).collect()) } );
    };

    (@munch ((num_args: $ex:expr) $($next:tt)*) -> {$($output:tt)*}) => {
        __cmd_opt!(@munch ($($next)*) -> { $($output)* (max_args: Some($ex)) (min_args: Some($ex)) } );
    };

    (@munch (($id:ident: $ex:expr) $($next:tt)*) -> {$($output:tt)*}) => {
        __cmd_opt!(@munch ($($next)*) -> { $($output)* ($id: $ex.into()) } );
    };

    (@munch $($input:tt)*) => {
        compile_error!("Could not parse command option.");
    };

    ($($id:ident: $ex:expr),*) => {
        __cmd_opt!(@munch ($(($id: $ex))*) -> {});
    };

}

/// ```
/// cmd!(Embed(_ctx, msg, args)
///     desc: "Add a new keyword embed.",
///     min_args: 2,
///     {
///         add_entry(&msg, &mut args, true)?
///     }
/// );
/// ```
/// ```
/// cmd!(Embed(_ctx, msg, args) desc: "Add a new keyword embed.", min_args: 2, {
///         add_entry(&msg, &mut args, true)?
///     }
/// );
/// ```
#[macro_export]
macro_rules! cmd {
    ($f:ident () $(,)? $($k:ident: $v:expr,)* $b:block) => {
        cmd!($f(_ctx, _msg, _args) $($k: $v,)* $b);
    };
    ($f:ident ($c:ident) $(,)? $($k:ident: $v:expr,)* $b:block) => {
        cmd!($f($c, _msg, _args) $($k: $v,)* $b);
    };
    ($f:ident ($c:ident, $m:ident) $(,)? $($k:ident: $v:expr,)* $b:block) => {
        cmd!($f($c, $m, _args) $($k: $v,)* $b);
    };
    ($f:ident ($c:ident, $m:ident, $a:ident) $(,)? $($k:ident: $v:expr,)* $b:block) => {
        pub struct $f { options: ::std::sync::Arc<::serenity::framework::standard::CommandOptions> }

        impl $f {
            fn new() -> $f {
                $f {
                    options: __cmd_opt!($($k: $v),*)
                }
            }
        }

        impl ::serenity::framework::standard::Command for $f {
            #[allow(unused_mut)]
            fn execute(&self,
                mut $c: &mut ::serenity::client::Context,
                $m: &::serenity::model::channel::Message,
                mut $a: ::serenity::framework::standard::Args,
            ) -> ::std::result::Result<(), ::serenity::framework::standard::CommandError> {
                $b

                Ok(())
            }

            fn options(&self) -> ::std::sync::Arc<::serenity::framework::standard::CommandOptions> {
                self.options.clone()
            }
        }
    };
}

#[macro_export]
macro_rules! grp {
    ($($cmd:ident),*$(,)?) => {
        pub fn commands(_: ::serenity::framework::standard::CreateGroup) -> ::serenity::framework::standard::CreateGroup {
            let mut g = ::serenity::framework::standard::CommandGroup::default();
            let c: &[std::sync::Arc<dyn serenity::framework::standard::Command>] = &[$(std::sync::Arc::new($cmd::new())),*];

            for command in c.iter() {
                let name = command.options().aliases[0].clone();

                for alias in command.options().aliases[1..].iter() {
                    g.commands.insert(alias.clone(), ::serenity::framework::standard::CommandOrAlias::Alias(name.clone()));
                }

                g.commands.insert(name, ::serenity::framework::standard::CommandOrAlias::Command(::std::sync::Arc::clone(command)));
            }

            ::serenity::framework::standard::CreateGroup(g)
        }
    };
}
