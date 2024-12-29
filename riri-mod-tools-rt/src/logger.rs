#![allow(dead_code, unused_macros)]
use std::sync::OnceLock;

/// Defines a color which can be used to set the color that a message will be printed as in the
/// Reloaded console output. Designed to closely represent C#'s Color type in
/// System.Drawing.Primitives:
/// https://github.com/dotnet/runtime/blob/main/src/libraries/System.Drawing.Primitives/src/System/Drawing/Color.cs
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LogColor(u32);

impl LogColor {
    pub const fn from_argb_u32(v: u32) -> Self { Self(v) }
    pub const fn from_argb_u8(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self::from_argb_u32(
            b as u32 | (g as u32) << 0x8 | 
            (r as u32) << 0x10 | (a as u32) << 0x18
        )
    }
    pub const fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self { 
        Self::from_argb_u8(0xff, r, g, b) 
    }
    pub fn get_red(&self) -> u8 { (self.0 >> 0x10) as u8 }
    pub fn get_green(&self) -> u8 { (self.0 >> 0x8) as u8 }
    pub fn get_blue(&self) -> u8 { self.0 as u8 }
    pub fn get_alpha(&self) -> u8 { (self.0 >> 0x18) as u8 }
}

/// See https://www.w3.org/TR/css-color-4/#named-colors
pub mod builtin_colors {
    pub const TRANSPARENT: super::LogColor = super::LogColor::from_argb_u32(0x00FFFFFF);
    pub const ALICEBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFFF0F8FF);
    pub const ANTIQUEWHITE: super::LogColor = super::LogColor::from_argb_u32(0xFFFAEBD7);
    pub const AQUA: super::LogColor = super::LogColor::from_argb_u32(0xFF00FFFF);
    pub const AQUAMARINE: super::LogColor = super::LogColor::from_argb_u32(0xFF7FFFD4);
    pub const AZURE: super::LogColor = super::LogColor::from_argb_u32(0xFFF0FFFF);
    pub const BEIGE: super::LogColor = super::LogColor::from_argb_u32(0xFFF5F5DC);
    pub const BISQUE: super::LogColor = super::LogColor::from_argb_u32(0xFFFFE4C4);
    pub const BLACK: super::LogColor = super::LogColor::from_argb_u32(0xFF000000);
    pub const BLANCHEDALMOND: super::LogColor = super::LogColor::from_argb_u32(0xFFFFEBCD);
    pub const BLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF0000FF);
    pub const BLUEVIOLET: super::LogColor = super::LogColor::from_argb_u32(0xFF8A2BE2);
    pub const BROWN: super::LogColor = super::LogColor::from_argb_u32(0xFFA52A2A);
    pub const BURLYWOOD: super::LogColor = super::LogColor::from_argb_u32(0xFFDEB887);
    pub const CADETBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF5F9EA0);
    pub const CHARTREUSE: super::LogColor = super::LogColor::from_argb_u32(0xFF7FFF00);
    pub const CHOCOLATE: super::LogColor = super::LogColor::from_argb_u32(0xFFD2691E);
    pub const CORAL: super::LogColor = super::LogColor::from_argb_u32(0xFFFF7F50);
    pub const CORNFLOWERBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF6495ED);
    pub const CORNSILK: super::LogColor = super::LogColor::from_argb_u32(0xFFFFF8DC);
    pub const CRIMSON: super::LogColor = super::LogColor::from_argb_u32(0xFFDC143C);
    pub const CYAN: super::LogColor = super::LogColor::from_argb_u32(0xFF00FFFF);
    pub const DARKBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF00008B);
    pub const DARKCYAN: super::LogColor = super::LogColor::from_argb_u32(0xFF008B8B);
    pub const DARKGOLDENROD: super::LogColor = super::LogColor::from_argb_u32(0xFFB8860B);
    pub const DARKGRAY: super::LogColor = super::LogColor::from_argb_u32(0xFFA9A9A9);
    pub const DARKGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF006400);
    pub const DARKKHAKI: super::LogColor = super::LogColor::from_argb_u32(0xFFBDB76B);
    pub const DARKMAGENTA: super::LogColor = super::LogColor::from_argb_u32(0xFF8B008B);
    pub const DARKOLIVEGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF556B2F);
    pub const DARKORANGE: super::LogColor = super::LogColor::from_argb_u32(0xFFFF8C00);
    pub const DARKORCHID: super::LogColor = super::LogColor::from_argb_u32(0xFF9932CC);
    pub const DARKRED: super::LogColor = super::LogColor::from_argb_u32(0xFF8B0000);
    pub const DARKSALMON: super::LogColor = super::LogColor::from_argb_u32(0xFFE9967A);
    pub const DARKSEAGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF8FBC8F);
    pub const DARKSLATEBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF483D8B);
    pub const DARKSLATEGRAY: super::LogColor = super::LogColor::from_argb_u32(0xFF2F4F4F);
    pub const DARKTURQUOISE: super::LogColor = super::LogColor::from_argb_u32(0xFF00CED1);
    pub const DARKVIOLET: super::LogColor = super::LogColor::from_argb_u32(0xFF9400D3);
    pub const DEEPPINK: super::LogColor = super::LogColor::from_argb_u32(0xFFFF1493);
    pub const DEEPSKYBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF00BFFF);
    pub const DIMGRAY: super::LogColor = super::LogColor::from_argb_u32(0xFF696969);
    pub const DODGERBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF1E90FF);
    pub const FIREBRICK: super::LogColor = super::LogColor::from_argb_u32(0xFFB22222);
    pub const FLORALWHITE: super::LogColor = super::LogColor::from_argb_u32(0xFFFFFAF0);
    pub const FORESTGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF228B22);
    pub const FUCHSIA: super::LogColor = super::LogColor::from_argb_u32(0xFFFF00FF);
    pub const GAINSBORO: super::LogColor = super::LogColor::from_argb_u32(0xFFDCDCDC);
    pub const GHOSTWHITE: super::LogColor = super::LogColor::from_argb_u32(0xFFF8F8FF);
    pub const GOLD: super::LogColor = super::LogColor::from_argb_u32(0xFFFFD700);
    pub const GOLDENROD: super::LogColor = super::LogColor::from_argb_u32(0xFFDAA520);
    pub const GRAY: super::LogColor = super::LogColor::from_argb_u32(0xFF808080);
    pub const GREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF008000);
    pub const GREENYELLOW: super::LogColor = super::LogColor::from_argb_u32(0xFFADFF2F);
    pub const HONEYDEW: super::LogColor = super::LogColor::from_argb_u32(0xFFF0FFF0);
    pub const HOTPINK: super::LogColor = super::LogColor::from_argb_u32(0xFFFF69B4);
    pub const INDIANRED: super::LogColor = super::LogColor::from_argb_u32(0xFFCD5C5C);
    pub const INDIGO: super::LogColor = super::LogColor::from_argb_u32(0xFF4B0082);
    pub const IVORY: super::LogColor = super::LogColor::from_argb_u32(0xFFFFFFF0);
    pub const KHAKI: super::LogColor = super::LogColor::from_argb_u32(0xFFF0E68C);
    pub const LAVENDER: super::LogColor = super::LogColor::from_argb_u32(0xFFE6E6FA);
    pub const LAVENDERBLUSH: super::LogColor = super::LogColor::from_argb_u32(0xFFFFF0F5);
    pub const LAWNGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF7CFC00);
    pub const LEMONCHIFFON: super::LogColor = super::LogColor::from_argb_u32(0xFFFFFACD);
    pub const LIGHTBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFFADD8E6);
    pub const LIGHTCORAL: super::LogColor = super::LogColor::from_argb_u32(0xFFF08080);
    pub const LIGHTCYAN: super::LogColor = super::LogColor::from_argb_u32(0xFFE0FFFF);
    pub const LIGHTGOLDENRODYELLOW: super::LogColor = super::LogColor::from_argb_u32(0xFFFAFAD2);
    pub const LIGHTGRAY: super::LogColor = super::LogColor::from_argb_u32(0xFFD3D3D3);
    pub const LIGHTGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF90EE90);
    pub const LIGHTPINK: super::LogColor = super::LogColor::from_argb_u32(0xFFFFB6C1);
    pub const LIGHTSALMON: super::LogColor = super::LogColor::from_argb_u32(0xFFFFA07A);
    pub const LIGHTSEAGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF20B2AA);
    pub const LIGHTSKYBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF87CEFA);
    pub const LIGHTSLATEGRAY: super::LogColor = super::LogColor::from_argb_u32(0xFF778899);
    pub const LIGHTSTEELBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFFB0C4DE);
    pub const LIGHTYELLOW: super::LogColor = super::LogColor::from_argb_u32(0xFFFFFFE0);
    pub const LIME: super::LogColor = super::LogColor::from_argb_u32(0xFF00FF00);
    pub const LIMEGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF32CD32);
    pub const LINEN: super::LogColor = super::LogColor::from_argb_u32(0xFFFAF0E6);
    pub const MAGENTA: super::LogColor = super::LogColor::from_argb_u32(0xFFFF00FF);
    pub const MAROON: super::LogColor = super::LogColor::from_argb_u32(0xFF800000);
    pub const MEDIUMAQUAMARINE: super::LogColor = super::LogColor::from_argb_u32(0xFF66CDAA);
    pub const MEDIUMBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF0000CD);
    pub const MEDIUMORCHID: super::LogColor = super::LogColor::from_argb_u32(0xFFBA55D3);
    pub const MEDIUMPURPLE: super::LogColor = super::LogColor::from_argb_u32(0xFF9370DB);
    pub const MEDIUMSEAGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF3CB371);
    pub const MEDIUMSLATEBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF7B68EE);
    pub const MEDIUMSPRINGGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF00FA9A);
    pub const MEDIUMTURQUOISE: super::LogColor = super::LogColor::from_argb_u32(0xFF48D1CC);
    pub const MEDIUMVIOLETRED: super::LogColor = super::LogColor::from_argb_u32(0xFFC71585);
    pub const MIDNIGHTBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF191970);
    pub const MINTCREAM: super::LogColor = super::LogColor::from_argb_u32(0xFFF5FFFA);
    pub const MISTYROSE: super::LogColor = super::LogColor::from_argb_u32(0xFFFFE4E1);
    pub const MOCCASIN: super::LogColor = super::LogColor::from_argb_u32(0xFFFFE4B5);
    pub const NAVAJOWHITE: super::LogColor = super::LogColor::from_argb_u32(0xFFFFDEAD);
    pub const NAVY: super::LogColor = super::LogColor::from_argb_u32(0xFF000080);
    pub const OLDLACE: super::LogColor = super::LogColor::from_argb_u32(0xFFFDF5E6);
    pub const OLIVE: super::LogColor = super::LogColor::from_argb_u32(0xFF808000);
    pub const OLIVEDRAB: super::LogColor = super::LogColor::from_argb_u32(0xFF6B8E23);
    pub const ORANGE: super::LogColor = super::LogColor::from_argb_u32(0xFFFFA500);
    pub const ORANGERED: super::LogColor = super::LogColor::from_argb_u32(0xFFFF4500);
    pub const ORCHID: super::LogColor = super::LogColor::from_argb_u32(0xFFDA70D6);
    pub const PALEGOLDENROD: super::LogColor = super::LogColor::from_argb_u32(0xFFEEE8AA);
    pub const PALEGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF98FB98);
    pub const PALETURQUOISE: super::LogColor = super::LogColor::from_argb_u32(0xFFAFEEEE);
    pub const PALEVIOLETRED: super::LogColor = super::LogColor::from_argb_u32(0xFFDB7093);
    pub const PAPAYAWHIP: super::LogColor = super::LogColor::from_argb_u32(0xFFFFEFD5);
    pub const PEACHPUFF: super::LogColor = super::LogColor::from_argb_u32(0xFFFFDAB9);
    pub const PERU: super::LogColor = super::LogColor::from_argb_u32(0xFFCD853F);
    pub const PINK: super::LogColor = super::LogColor::from_argb_u32(0xFFFFC0CB);
    pub const PLUM: super::LogColor = super::LogColor::from_argb_u32(0xFFDDA0DD);
    pub const POWDERBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFFB0E0E6);
    pub const PURPLE: super::LogColor = super::LogColor::from_argb_u32(0xFF800080);
    pub const REBECCAPURPLE: super::LogColor = super::LogColor::from_argb_u32(0xFF663399);
    pub const RED: super::LogColor = super::LogColor::from_argb_u32(0xFFFF0000);
    pub const ROSYBROWN: super::LogColor = super::LogColor::from_argb_u32(0xFFBC8F8F);
    pub const ROYALBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF4169E1);
    pub const SADDLEBROWN: super::LogColor = super::LogColor::from_argb_u32(0xFF8B4513);
    pub const SALMON: super::LogColor = super::LogColor::from_argb_u32(0xFFFA8072);
    pub const SANDYBROWN: super::LogColor = super::LogColor::from_argb_u32(0xFFF4A460);
    pub const SEAGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF2E8B57);
    pub const SEASHELL: super::LogColor = super::LogColor::from_argb_u32(0xFFFFF5EE);
    pub const SIENNA: super::LogColor = super::LogColor::from_argb_u32(0xFFA0522D);
    pub const SILVER: super::LogColor = super::LogColor::from_argb_u32(0xFFC0C0C0);
    pub const SKYBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF87CEEB);
    pub const SLATEBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF6A5ACD);
    pub const SLATEGRAY: super::LogColor = super::LogColor::from_argb_u32(0xFF708090);
    pub const SNOW: super::LogColor = super::LogColor::from_argb_u32(0xFFFFFAFA);
    pub const SPRINGGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF00FF7F);
    pub const STEELBLUE: super::LogColor = super::LogColor::from_argb_u32(0xFF4682B4);
    pub const TAN: super::LogColor = super::LogColor::from_argb_u32(0xFFD2B48C);
    pub const TEAL: super::LogColor = super::LogColor::from_argb_u32(0xFF008080);
    pub const THISTLE: super::LogColor = super::LogColor::from_argb_u32(0xFFD8BFD8);
    pub const TOMATO: super::LogColor = super::LogColor::from_argb_u32(0xFFFF6347);
    pub const TURQUOISE: super::LogColor = super::LogColor::from_argb_u32(0xFF40E0D0);
    pub const VIOLET: super::LogColor = super::LogColor::from_argb_u32(0xFFEE82EE);
    pub const WHEAT: super::LogColor = super::LogColor::from_argb_u32(0xFFF5DEB3);
    pub const WHITE: super::LogColor = super::LogColor::from_argb_u32(0xFFFFFFFF);
    pub const WHITESMOKE: super::LogColor = super::LogColor::from_argb_u32(0xFFF5F5F5);
    pub const YELLOW: super::LogColor = super::LogColor::from_argb_u32(0xFFFFFF00);
    pub const YELLOWGREEN: super::LogColor = super::LogColor::from_argb_u32(0xFF9ACD32);
}

/// Defines levels 
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Verbose,
    Debug,
    Information,
    Warning,
    Error
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verbose => write!(f, "TRACE"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Information => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR")
        }
    }
}

impl LogLevel {
    pub fn get_log_color(&self) -> LogColor {
        match self {
            Self::Verbose |
            Self::Debug |
            Self::Information => GLOBAL_LOG_COLOR,
            Self::Warning => builtin_colors::SANDYBROWN,
            Self::Error => builtin_colors::RED
        }
    }
}

pub static GLOBAL_LOG_COLOR: LogColor = builtin_colors::LIMEGREEN;
pub static GLOBAL_LOG_LEVEL: LogLevel = LogLevel::Verbose;

#[macro_export]
macro_rules! log {
    ($ty:ident, $($fmt:tt)*) => {
        if $crate::logger::LogLevel::$ty as u32 >= $crate::logger::GLOBAL_LOG_LEVEL as u32 {
            let text: String = format!($($fmt)*);
            unsafe {
                $crate::logger::invoke_reloaded_logger(text.as_ptr(), text.len(), $crate::logger::GLOBAL_LOG_LEVEL.get_log_color());
            }
        }
    };
    ($ty:ident, $col:path, $($fmt:tt)*) => {
        if $crate::logger::LogLevel::$ty as u32 >= $crate::logger::GLOBAL_LOG_LEVEL as u32 {
            let text: String = format!($($fmt)*);
            unsafe {
                $crate::logger::invoke_reloaded_logger(text.as_ptr(), text.len(), $crate::logger::builtin_colors::$col);
            }
        }
    };
}
#[macro_export]
macro_rules! logln {
    ($ty:ident, $($fmt:tt)*) => {
        if $crate::logger::LogLevel::$ty as u32 >= $crate::logger::GLOBAL_LOG_LEVEL as u32 {
            let text: String = format!($($fmt)*);
            // text
            unsafe {
                $crate::logger::invoke_reloaded_logger_newline(text.as_ptr(), text.len(), $crate::logger::GLOBAL_LOG_LEVEL.get_log_color());
            }
        }
    };
    ($ty:ident, $col:path, $($fmt:tt)*) => {
        if $crate::logger::LogLevel::$ty as u32 >= $crate::logger::GLOBAL_LOG_LEVEL as u32 {
            let text: String = format!($($fmt)*);
            unsafe {
                $crate::logger::invoke_reloaded_logger_newline(text.as_ptr(), text.len(), $crate::logger::builtin_colors::$col);
            }
        }
    };
}


type LogFn = unsafe extern "C" fn(*const u8, usize, LogColor) -> ();
/// A function pointer to invoke WriteAsync method in Reloaded-II's logger. This allows for
/// us to write into the console output and have that saved into a log file.
pub static RELOADED_LOGGER: OnceLock<LogFn> = OnceLock::new();
/// A function pointer to invoke WriteLineAsync method in Reloaded-II's logger. This allows for
/// us to write into the console output and have that saved into a log file.
pub static RELOADED_LOGGER_LN: OnceLock<LogFn> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_reloaded_logger(cb: LogFn) {
    RELOADED_LOGGER.set(cb).unwrap();    
}

#[no_mangle]
pub unsafe extern "C" fn set_reloaded_logger_newline(cb: LogFn) {
    RELOADED_LOGGER_LN.set(cb).unwrap();
    logln!(Information, "Logger connected! :3");
}

#[no_mangle]
pub unsafe extern "C" fn invoke_reloaded_logger(p: *const u8, len: usize, c: LogColor) {
    (RELOADED_LOGGER.get().unwrap())(p, len, c);
}

#[no_mangle]
pub unsafe extern "C" fn invoke_reloaded_logger_newline(p: *const u8, len: usize, c: LogColor) {
    (RELOADED_LOGGER_LN.get().unwrap())(p, len, c);
}
