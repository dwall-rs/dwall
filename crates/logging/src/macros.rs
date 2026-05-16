/// 内部处理宏，递归解析 tracing 风格的键值对，并与主消息分离传递
#[macro_export]
macro_rules! kv_log_internal {
    // 基础情况 1：不带 target
    (@build $lvl:ident () ($($kv_fmt:tt)*) ($($kv_args:expr,)*) $msg:literal $(, $($msg_args:tt)*)?) => {
        ::log::$lvl!(
            concat!("{} ", $($kv_fmt)*),
            format_args!($msg $(, $($msg_args)*)?),
            $($kv_args,)*
        )
    };

    // 基础情况 2：带 target
    (@build $lvl:ident (target: $tgt:expr) ($($kv_fmt:tt)*) ($($kv_args:expr,)*) $msg:literal $(, $($msg_args:tt)*)?) => {
        ::log::$lvl!(
            target: $tgt,
            concat!("{} ", $($kv_fmt)*),
            format_args!($msg $(, $($msg_args)*)?),
            $($kv_args,)*
        )
    };

    // 解析带 % 前缀的键值对（Display）
    (@build $lvl:ident $tgt:tt ($($kv_fmt:tt)*) ($($kv_args:expr,)*) $k:ident = %$v:expr, $($rest:tt)+) => {
        $crate::kv_log_internal!(@build $lvl $tgt
            ($($kv_fmt)* stringify!($k), "={} ",)
            ($($kv_args,)* $v,)
            $($rest)+
        )
    };

    // 解析带 ? 前缀的键值对（Debug）
    (@build $lvl:ident $tgt:tt ($($kv_fmt:tt)*) ($($kv_args:expr,)*) $k:ident = ?$v:expr, $($rest:tt)+) => {
        $crate::kv_log_internal!(@build $lvl $tgt
            ($($kv_fmt)* stringify!($k), "={:?} ",)
            ($($kv_args,)* $v,)
            $($rest)+
        )
    };

    // 解析无前缀的键值对（退化到 Debug）
    (@build $lvl:ident $tgt:tt ($($kv_fmt:tt)*) ($($kv_args:expr,)*) $k:ident = $v:expr, $($rest:tt)+) => {
        $crate::kv_log_internal!(@build $lvl $tgt
            ($($kv_fmt)* stringify!($k), "={:?} ",)
            ($($kv_args,)* $v,)
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
