#[macro_export]
macro_rules! widgets {
    (
        $name:ident;
        $(
            $widget:ident:
            $(
                [$mode:pat_param]
                =>
            )?
            $struc:ident,
        )+
        [popups]: {
            $(
                $pwidget:ident:
                $(
                    [$pmode:pat_param]
                    =>
                )?
                $pstruc:ident,
            )+
        }
    ) => {
        #[derive(Default)]
        pub struct $name {
        $(
            pub $widget: $struc,
        )+
        $(
            pub $pwidget: $pstruc,
        )+
        }

        impl $name {
            fn draw_popups(&mut self, ctx: &$crate::app::Context, f: &mut ratatui::Frame) {
                match ctx.mode {
                    $(
                        $($pmode => self.$pwidget.draw(f, ctx, f.size()),)?
                    )+
                    _ => {}
                }

            }

            fn get_help(&self, mode: &$crate::app::Mode) -> Option<Vec<(&'static str, &'static str)>> {
                match mode {
                    $(
                        $($mode => $struc::get_help(),)?
                    )+
                    $(
                        $($pmode => $pstruc::get_help(),)?
                    )+
                    _ => None,
                }
            }

            fn handle_event(&mut self, ctx: &mut $crate::app::Context, evt: &crossterm::event::Event) {
                match ctx.mode {
                    $(
                        $($mode => self.$widget.handle_event(ctx, evt),)?
                    )+
                    $(
                        $($pmode => self.$pwidget.handle_event(ctx, evt),)?
                    )+
                    _ => {}
                };
            }
        }
    }
}

#[macro_export]
macro_rules! cats {
    (
        $(
            $cats:expr => {$($idx:expr => ($icon:expr, $disp:expr, $conf:expr, $col:tt$(.$colext:tt)*);)+}
        )+
    ) => {{
        let v = vec![
        $(
            $crate::widget::category::CatStruct {
                name: $cats.to_string(),
                entries: vec![$($crate::widget::category::CatEntry::new(
                        $disp,
                        $conf,
                        $idx,
                        $icon,
                        |theme: &$crate::theme::Theme| {theme.$col$(.$colext)*},
                    ),
                )+],
        },)+
        ];
        v
    }}
}

#[macro_export]
macro_rules! popup_enum {
    (
        $name:ident;
        $(
            $(#[$docs:meta])*
            ($num:expr, $konst:ident, $phrase:expr);
        )+
    ) => {
        #[derive(PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
        pub enum $name {
        $(
            $(#[$docs])*
            $konst = $num,
        )+
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                    $(
                        $name::$konst => $phrase,
                    )+
                };
                write!(f, "{}", s)
            }
        }

        impl $crate::widget::EnumIter<$name> for $name {
            fn iter() -> std::slice::Iter<'static, $name> {
                static ITEMS: &[$name] = &[
                    $(
                        $name::$konst,
                    )+
                ];
                ITEMS.iter()
            }
        }

        impl TryFrom<usize> for $name {
            type Error = ();

            fn try_from(v: usize) -> Result<Self, Self::Error> {
                match v {
                    $(
                        x if x == $name::$konst as usize => Ok($name::$konst),
                    )+
                    _ => Err(()),
                }
            }
        }
    }
}

#[macro_export]
macro_rules! style {
    (
        $($method:ident$(:$value:expr)?),* $(,)?
    ) => {{
            #[allow(unused_imports)]
            use ratatui::style::Stylize;

            let style = ratatui::style::Style::new()
                $(.$method($($value)?))?;
            style
    }};
}

#[macro_export]
macro_rules! title {
    // Single input
    ($arg:expr) => {{
        let res = format!("{}", $arg);
        res
    }};

    // format-like
    ($($arg:expr),*$(,)?) => {{
        let res = format!("{}", format!($($arg),*));
        res
    }};

    // vec-like
    ($($arg:expr);*$(;)?) => {{
        let res = vec![
            $($arg,)*
        ];
        res
    }};
}

#[macro_export]
macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        core::convert::From::from([$($v,)*])
    }};
}

#[macro_export]
macro_rules! raw {
    (
        $text:expr
    ) => {{
        let raw = Text::raw($text);
        raw
    }};
}

#[macro_export]
macro_rules! cond_vec {
    ($($cond:expr => $x:expr),+ $(,)?) => {{
        let v = vec![$(($cond, $x)),*].iter().filter_map(|(c, x)| {
            if *c { Some(x.to_owned()) } else { None }
        }).collect::<Vec<_>>();
        v
    }};
    ($cond:expr ; $x:expr) => {{
        let v = $cond
            .iter()
            .zip($x)
            .filter_map(|(c, val)| if *c { Some(val.to_owned()) } else { None })
            .collect::<Vec<_>>();
        v
    }};
}

#[macro_export]
macro_rules! sel {
    (
        $text:expr
    ) => {{
        let raw = Selector::parse($text).map_err(|e| e.to_string());
        raw
    }};
}
