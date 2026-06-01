/// Internal processing macro that recursively parses tracing-style key-value pairs.
///
/// Output format:  `{message}, {key1}={val1}, {key2}={val2}, ...`
/// In debug builds, each key is rendered with ANSI bold-italic styling.
#[macro_export]
macro_rules! kv_log_internal {
    // ── Base case 1: no target ───────────────────────────────────────────────
    // Accumulated kv segments are each prefixed with ", " so the final string is:
    //   "{msg}, k1=v1, k2=v2"
    (@build $lvl:ident () ($($kv_fmt:tt)*) ($($kv_args:expr,)*) $msg:literal $(, $($msg_args:tt)*)?) => {
        ::log::$lvl!(
            concat!("{}", $($kv_fmt)*),
            format_args!($msg $(, $($msg_args)*)?),
            $($kv_args,)*
        )
    };

    // ── Base case 2: with target ─────────────────────────────────────────────
    (@build $lvl:ident (target: $tgt:expr) ($($kv_fmt:tt)*) ($($kv_args:expr,)*) $msg:literal $(, $($msg_args:tt)*)?) => {
        ::log::$lvl!(
            target: $tgt,
            concat!("{}", $($kv_fmt)*),
            format_args!($msg $(, $($msg_args)*)?),
            $($kv_args,)*
        )
    };

    // ── key = %value  (Display) ──────────────────────────────────────────────
    (@build $lvl:ident $tgt:tt ($($kv_fmt:tt)*) ($($kv_args:expr,)*) $k:ident = %$v:expr, $($rest:tt)+) => {
        $crate::kv_log_internal!(@build $lvl $tgt
            ($($kv_fmt)* ", {}={}",)
            ($($kv_args,)* $crate::__fmt_key(::core::stringify!($k)), $v,)
            $($rest)+
        )
    };

    // ── key = ?value  (Debug) ────────────────────────────────────────────────
    (@build $lvl:ident $tgt:tt ($($kv_fmt:tt)*) ($($kv_args:expr,)*) $k:ident = ?$v:expr, $($rest:tt)+) => {
        $crate::kv_log_internal!(@build $lvl $tgt
            ($($kv_fmt)* ", {}={:?}",)
            ($($kv_args,)* $crate::__fmt_key(::core::stringify!($k)), $v,)
            $($rest)+
        )
    };

    // ── key = value  (Debug fallback) ────────────────────────────────────────
    (@build $lvl:ident $tgt:tt ($($kv_fmt:tt)*) ($($kv_args:expr,)*) $k:ident = $v:expr, $($rest:tt)+) => {
        $crate::kv_log_internal!(@build $lvl $tgt
            ($($kv_fmt)* ", {}={:?}",)
            ($($kv_args,)* $crate::__fmt_key(::core::stringify!($k)), $v,)
            $($rest)+
        )
    };
}

#[macro_export]
macro_rules! error {
    (target: $tgt:expr, $($args:tt)+) => {
        $crate::kv_log_internal!(@build error (target: $tgt) () () $($args)+)
    };
    ($($args:tt)+) => {
        $crate::kv_log_internal!(@build error () () () $($args)+)
    };
}

#[macro_export]
macro_rules! warn {
    (target: $tgt:expr, $($args:tt)+) => {
        $crate::kv_log_internal!(@build warn (target: $tgt) () () $($args)+)
    };
    ($($args:tt)+) => {
        $crate::kv_log_internal!(@build warn () () () $($args)+)
    };
}

#[macro_export]
macro_rules! info {
    (target: $tgt:expr, $($args:tt)+) => {
        $crate::kv_log_internal!(@build info (target: $tgt) () () $($args)+)
    };
    ($($args:tt)+) => {
        $crate::kv_log_internal!(@build info () () () $($args)+)
    };
}

#[macro_export]
macro_rules! debug {
    (target: $tgt:expr, $($args:tt)+) => {
        if !$crate::LOG_MAX_LEVEL_INFO {
            $crate::kv_log_internal!(@build debug (target: $tgt) () () $($args)+)
        }
    };
    ($($args:tt)+) => {
        if !$crate::LOG_MAX_LEVEL_INFO {
            $crate::kv_log_internal!(@build debug () () () $($args)+)
        }
    };
}

#[macro_export]
macro_rules! trace {
    (target: $tgt:expr, $($args:tt)+) => {
        if !$crate::LOG_MAX_LEVEL_INFO {
            $crate::kv_log_internal!(@build trace (target: $tgt) () () $($args)+)
        }
    };
    ($($args:tt)+) => {
        if !$crate::LOG_MAX_LEVEL_INFO {
            $crate::kv_log_internal!(@build trace () () () $($args)+)
        }
    };
}
