macro_rules! config_rescuable {
    ($param:expr, $t:ty) => (
        macro_rules! begin_rescuable {
            ($i:ident) => {
                let mut $i = |v: $t| {};
            }
        }

        macro_rules! on_rescue {
            (|$v:ident| $e:expr, $j:ident) => (
                let mut $j = |$v: $t| {
                    $e;
                    $j($v);
                };
            )
        }

        macro_rules! trr {
            ($e:expr, $j:ident) => (match $e {
                Ok(val) => val,
                Err(err) => {
                    $j($param);
                    return Err(err);
                }
            });
        }

        macro_rules! end_rescuable {
            ($j:ident) => (
                ::std::mem::drop($j);
            )
        }
    )
}
