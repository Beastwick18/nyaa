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
            $(&$cat, )+
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
