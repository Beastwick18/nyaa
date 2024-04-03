#[macro_export]
macro_rules! categories {
    (
        $name:ident;
        $(
            ($cat:ident: $cats:expr) => {$($idx:expr => ($icon:expr, $disp:expr, $conf:expr, $col:ident);)+}
        )+
    ) => {
        $(
            pub static $cat: CatStruct = CatStruct {
                name: $cats,
                entries: &[$(CatEntry::new(
                        $disp,
                        $conf,
                        $idx,
                        $icon,
                        Color::$col,
                    ),
                )+],
            };
        )+

        pub static $name: &[&CatStruct] = &[
            $(&$cat,)+
        ];
    }
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
        #[derive(Clone, Copy, Serialize, Deserialize)]
        pub enum $name {
        $(
            $(#[$docs])*
            $konst = $num,
        )+
        }
        impl ToString for $name {
            fn to_string(&self) -> String {
                {
                    match self {
                        $(
                            $name::$konst => $phrase.to_owned(),
                        )+
                    }
                }
            }
        }


        impl EnumIter<$name> for $name {
            fn iter() -> std::slice::Iter<'static, $name> {
                static ITEMS: &[$name] = &[
                    $(
                        $name::$konst,
                    )+
                ];
                ITEMS.iter()
            }
        }
    }
}

#[macro_export]
macro_rules! style {
    (
        $($method:ident$(:$value:expr)?),* $(,)?
    ) => {
        {
            #[allow(unused_imports)]
            use ratatui::style::Stylize;
            let style = ratatui::style::Style::new()
                $(.$method($($value)?))?;
            style
        }
    };
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
    ($($cond:expr => $x:expr),+ $(,)?) => {
        {
            let mut v = vec![];
            $(if $cond { v.push($x); })*
            v
        }
    };
    ($cond:expr ; $x:expr) => {
        {
            let mut v = vec![];
            for (c, val) in $cond.iter().zip($x) {
                if *c {
                    v.push(val.to_owned());
                }
            }
            v
        }
    };
}
