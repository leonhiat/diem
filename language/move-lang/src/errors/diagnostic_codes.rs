// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//**************************************************************************************************
// Main types
//**************************************************************************************************

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Severity {
    BlockingError = 0,
    NonblockingError = 1,
    Warning = 2,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub(crate) struct DiagnosticInfo {
    pub(crate) severity: Severity,
    category: Category,
    code: u8,
    message: &'static str,
}

pub(crate) trait DiagnosticCode: Copy {
    const CATEGORY: Category;

    fn severity(self) -> Severity;

    fn code_and_message(self) -> (u8, &'static str);

    fn into_info(self) -> DiagnosticInfo {
        let severity = self.severity();
        let category = Self::CATEGORY;
        let (code, message) = self.code_and_message();
        DiagnosticInfo {
            severity,
            category,
            code,
            message,
        }
    }
}

//**************************************************************************************************
// Categories and Codes
//**************************************************************************************************

macro_rules! codes {
    ($($cat:ident: [
        $($code:ident: { msg: $code_msg:literal, severity:$sev:ident $(,)? }),* $(,)?
    ]),* $(,)?) => {
        #[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
        #[repr(u8)]
        pub enum Category {
            $($cat,)*
        }

        $(
            #[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
            #[repr(u8)]
            pub enum $cat {
                DontStartAtZeroPlaceholder,
                $($code,)*
            }

            impl DiagnosticCode for $cat {
                const CATEGORY: Category = {
                    // hacky check that $cat_num <= 99
                    let cat_is_leq_99 = (Category::$cat as u8) <= 99;
                    ["Diagnostic Category must be a u8 <= 99"][!cat_is_leq_99 as usize];
                    Category::$cat
                };

                fn severity(self) -> Severity {
                    match self {
                        Self::DontStartAtZeroPlaceholder =>
                            panic!("ICE do not use placeholder error code"),
                        $(Self::$code => Severity::$sev,)*
                    }
                }

                fn code_and_message(self) -> (u8, &'static str) {
                    let code = self as u8;
                    debug_assert!(code > 0);
                    match self {
                        Self::DontStartAtZeroPlaceholder =>
                            panic!("ICE do not use placeholder error code"),
                        $(Self::$code => (code, $code_msg),)*
                    }
                }
            }
        )*

    };
}

codes!(
    // bucket for random one off errors. unlikely to be used
    Uncategorized: [],
    // syntax errors
    Syntax: [
        InvalidNumber: { msg: "invalid number literal", severity: BlockingError },
        InvalidByteString: { msg: "invalid byte string", severity: BlockingError },
        InvalidHexString: { msg: "invalid hex string", severity: BlockingError },
        InvalidLValue: { msg: "invalid assignment", severity: BlockingError },
        SpecContextRestricted: { msg: "syntax item restricted to spec contexts", severity: BlockingError },
    ],
    // errors for any rules around declaration items
    Declarations: [
        DuplicateItem: { msg: "duplicate declaration, item, or annotation", severity: NonblockingError },
        UnnecessaryItem: { msg: "unnecessary or extraneous item", severity: NonblockingError },
        InvalidAddress: { msg: "invalid 'address' declaration", severity: NonblockingError },
        InvalidModule: { msg: "invalid 'module' declaration", severity: NonblockingError },
        InvalidScript: { msg: "invalid 'script' declaration", severity: NonblockingError },
        InvalidConstant: { msg: "invalid 'const' declaration", severity: NonblockingError },
        InvalidFunction: { msg: "invalid 'fun' declaration", severity: NonblockingError },
        InvalidStruct: { msg: "invalid 'struct' declaration", severity: NonblockingError },
        InvalidName: { msg: "invalid name", severity: BlockingError },
        InvalidFriendDeclaration: { msg: "invalid 'friend' declaration", severity: NonblockingError },
        InvalidAcquiresItem: { msg: "invalid 'acquires' item", severity: NonblockingError },
    ],
    // errors name resolution, mostly expansion/translate and naming/translate
    NameResolution: [
        UnboundAddress: { msg: "unbound address", severity: BlockingError },
        UnboundModule: { msg: "unbound module", severity: BlockingError },
        UnboundModuleMember: { msg: "unbound module member", severity: BlockingError },
        UnboundType: { msg: "unbound type", severity: BlockingError },
        UnboundUnscopedName: { msg: "unbound unscoped name", severity: BlockingError },
        NamePositionMismatch: { msg: "unexpected name in this position", severity: BlockingError },
        TooManyTypeArguments: { msg: "too many type arguments", severity: NonblockingError },
        TooFewTypeArguments: { msg: "too few type arguments", severity: BlockingError },
    ],
    // errors for typing rules. mostly typing/translate
    TypeSafety: [],
    // errors for ability rules. mostly typing/translate
    AbilitySafety: [],
    // errors for move rules. mostly cfgir/locals
    MoveSafety: [],
    // errors for move rules. mostly cfgir/borrows
    ReferenceSafety: [],
    // errors for any unused code or items
    UnusedItem: [
        Alias: { msg: "unused alias", severity: Warning },
    ],
);

//**************************************************************************************************
// impls
//**************************************************************************************************

impl DiagnosticInfo {
    pub fn render(self) -> (/* code */ String, /* message */ &'static str) {
        let Self {
            severity,
            category,
            code,
            message,
        } = self;
        let sev_prefix = match severity {
            Severity::BlockingError | Severity::NonblockingError => "E",
            Severity::Warning => "W",
        };
        let cat_prefix: u8 = category as u8;
        debug_assert!(cat_prefix <= 99);
        let string_code = format!("{}{:02}{:03}", sev_prefix, cat_prefix, code);
        (string_code, message)
    }
}

impl Severity {
    pub fn into_codespan_severity(self) -> codespan_reporting_new::diagnostic::Severity {
        use codespan_reporting_new::diagnostic::Severity as CSRSeverity;
        match self {
            Severity::BlockingError | Severity::NonblockingError => CSRSeverity::Error,
            Severity::Warning => CSRSeverity::Warning,
        }
    }
}
