// Written by `tests/js/format-tables.mjs` from ICU 78.2 (Node 22.23.0): do not
// edit by hand — run it again. `tests/build_and_runtime_agree.rs` holds the
// build's formatting to the browser's `Intl` for these locales.

static NUMBERS_0: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "A$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("PHP", "₱"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#K", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#M", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#B", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#T", forms: &[] }
    ],
};

static DATES_1: Dates = Dates {
    date: ["M/d/yy", "MMM d, yyyy", "MMMM d, yyyy", "EEEE, MMMM d, yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a' Coordinated Universal Time'"],
    datetime: ["M/d/yy, h:mm a", "MMM d, yyyy, h:mm a", "MMMM d, yyyy' at 'h:mm a", "EEEE, MMMM d, yyyy' at 'h:mm a"],
    months: [["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"], ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]],
    weekdays: ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
    periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
    standalone_months: [["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"], ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]],
    standalone_weekdays: [["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"], ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_2: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "A$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("PHP", "₱"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#k", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#m", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#bn", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#tn", forms: &[] }
    ],
};

static DATES_3: Dates = Dates {
    date: ["dd/MM/yyyy", "d MMM yyyy", "d MMMM yyyy", "EEEE, d MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' Coordinated Universal Time'"],
    datetime: ["dd/MM/yyyy, HH:mm", "d MMM yyyy, HH:mm", "d MMMM yyyy' at 'HH:mm", "EEEE, d MMMM yyyy' at 'HH:mm"],
    months: [["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"], ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sept", "Oct", "Nov", "Dec"]],
    weekdays: ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"], ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sept", "Oct", "Nov", "Dec"]],
    standalone_weekdays: [["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"], ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]],
    standalone_periods: ["am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm"],
};

static NUMBERS_4: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("AUD", "$")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#K", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#M", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#B", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#T", forms: &[] }
    ],
};

static DATES_5: Dates = Dates {
    date: ["d/M/yy", "d MMM yyyy", "d MMMM yyyy", "EEEE, d MMMM yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a' Coordinated Universal Time'"],
    datetime: ["d/M/yy, h:mm a", "d MMM yyyy, h:mm a", "d MMMM yyyy' at 'h:mm a", "EEEE, d MMMM yyyy' at 'h:mm a"],
    months: [["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"], ["Jan", "Feb", "Mar", "Apr", "May", "June", "July", "Aug", "Sept", "Oct", "Nov", "Dec"]],
    weekdays: ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
    periods: ["am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm"],
    standalone_months: [["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"], ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sept", "Oct", "Nov", "Dec"]],
    standalone_weekdays: [["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"], ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]],
    standalone_periods: ["am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm"],
};

static NUMBERS_6: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "$"), ("AUD", "A$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("PHP", "₱"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#K", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#M", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#B", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#T", forms: &[] }
    ],
};

static DATES_7: Dates = Dates {
    date: ["yyyy-MM-dd", "MMM d, yyyy", "MMMM d, yyyy", "EEEE, MMMM d, yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a' Coordinated Universal Time'"],
    datetime: ["yyyy-MM-dd, h:mm a", "MMM d, yyyy, h:mm a", "MMMM d, yyyy' at 'h:mm a", "EEEE, MMMM d, yyyy' at 'h:mm a"],
    months: [["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"], ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]],
    weekdays: ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
    periods: ["a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m."],
    standalone_months: [["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"], ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]],
    standalone_weekdays: [["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"], ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]],
    standalone_periods: ["a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m."],
};

static NUMBERS_8: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: true,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "A$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("PHP", "₱"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#K", forms: &[] },
        Compact { magnitude: 5, scale: 5, other: "#L", forms: &[] },
        Compact { magnitude: 7, scale: 7, other: "#Cr", forms: &[] },
        Compact { magnitude: 10, scale: 10, other: "#KCr", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#LCr", forms: &[] }
    ],
};

static DATES_9: Dates = Dates {
    date: ["dd/MM/yy", "d MMM yyyy", "d MMMM yyyy", "EEEE, d MMMM yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a' Coordinated Universal Time'"],
    datetime: ["dd/MM/yy, h:mm a", "d MMM yyyy, h:mm a", "d MMMM yyyy' at 'h:mm a", "EEEE, d MMMM yyyy' at 'h:mm a"],
    months: [["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"], ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sept", "Oct", "Nov", "Dec"]],
    weekdays: ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
    periods: ["am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm"],
    standalone_months: [["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"], ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sept", "Oct", "Nov", "Dec"]],
    standalone_weekdays: [["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"], ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]],
    standalone_periods: ["am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm"],
};

static NUMBERS_10: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}Mio.", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}Mrd.", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}Bio.", forms: &[] }
    ],
};

static DATES_11: Dates = Dates {
    date: ["dd.MM.yy", "dd.MM.yyyy", "d. MMMM yyyy", "EEEE, d. MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' Koordinierte Weltzeit'"],
    datetime: ["dd.MM.yy, HH:mm", "dd.MM.yyyy, HH:mm", "d. MMMM yyyy' um 'HH:mm", "EEEE, d. MMMM yyyy' um 'HH:mm"],
    months: [["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"], ["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"]],
    weekdays: ["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"], ["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"]],
    standalone_weekdays: [["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"], ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_12: Numbers = Numbers {
    zero: '0',
    group: "\u{a0}",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "¤\u{a0}#",
    currency_negative: "-¤\u{a0}#",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}Mio.", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}Mrd.", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}Bio.", forms: &[] }
    ],
};

static DATES_13: Dates = Dates {
    date: ["dd.MM.yy", "dd.MM.yyyy", "d. MMMM yyyy", "EEEE, d. MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' Koordinierte Weltzeit'"],
    datetime: ["dd.MM.yy, HH:mm", "dd.MM.yyyy, HH:mm", "d. MMMM yyyy' um 'HH:mm", "EEEE, d. MMMM yyyy' um 'HH:mm"],
    months: [["Jänner", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"], ["Jän", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"]],
    weekdays: ["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["Jänner", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"], ["Jän", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"]],
    standalone_weekdays: [["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"], ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_14: Numbers = Numbers {
    zero: '0',
    group: "'",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤\u{a0}#",
    currency_negative: "¤-#",
    currency_group: "'",
    currency_decimal: ".",
    symbols: &[("USD", "$"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}Mio.", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}Mrd.", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}Bio.", forms: &[] }
    ],
};

static NUMBERS_15: Numbers = Numbers {
    zero: '0',
    group: "\u{202f}",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{202f}",
    currency_decimal: ",",
    symbols: &[("USD", "$US"), ("EUR", "€"), ("GBP", "£GB"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "$CA"), ("AUD", "$AU"), ("BRL", "R$"), ("MXN", "$MX"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}k", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}M", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}Md", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}Bn", forms: &[] }
    ],
};

static DATES_16: Dates = Dates {
    date: ["dd/MM/yyyy", "d MMM yyyy", "d MMMM yyyy", "EEEE d MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' temps universel coordonné'"],
    datetime: ["dd/MM/yyyy HH:mm", "d MMM yyyy, HH:mm", "d MMMM yyyy à HH:mm", "EEEE d MMMM yyyy à HH:mm"],
    months: [["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"], ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."]],
    weekdays: ["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"], ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."]],
    standalone_weekdays: [["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"], ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_17: Numbers = Numbers {
    zero: '0',
    group: "\u{a0}",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{a0}",
    currency_decimal: ",",
    symbols: &[("USD", "$\u{a0}US"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥"), ("CAD", "$"), ("AUD", "$\u{a0}AU"), ("BRL", "R$")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}k", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}M", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}G", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}T", forms: &[] }
    ],
};

static DATES_18: Dates = Dates {
    date: ["yyyy-MM-dd", "d MMM yyyy", "d MMMM yyyy", "EEEE d MMMM yyyy"],
    time: ["HH' h 'mm", "HH' h 'mm' min 'ss' s'", "HH' h 'mm' min 'ss' s UTC'", "HH' h 'mm' min 'ss' s temps universel coordonné'"],
    datetime: ["yyyy-MM-dd HH' h 'mm", "d MMM yyyy, HH' h 'mm", "d MMMM yyyy à HH' h 'mm", "EEEE d MMMM yyyy à HH' h 'mm"],
    months: [["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"], ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juill.", "août", "sept.", "oct.", "nov.", "déc."]],
    weekdays: ["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"], ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juill.", "août", "sept.", "oct.", "nov.", "déc."]],
    standalone_weekdays: [["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"], ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."]],
    standalone_periods: ["a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m."],
};

static NUMBERS_19: Numbers = Numbers {
    zero: '0',
    group: "\u{202f}",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{202f}",
    currency_decimal: ".",
    symbols: &[("USD", "$US"), ("EUR", "€"), ("GBP", "£GB"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "$CA"), ("AUD", "$AU"), ("BRL", "R$"), ("MXN", "$MX"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}k", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}M", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}Md", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}Bn", forms: &[] }
    ],
};

static DATES_20: Dates = Dates {
    date: ["dd.MM.yy", "d MMM yyyy", "d MMMM yyyy", "EEEE, d MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH.mm:ss' h temps universel coordonné'"],
    datetime: ["dd.MM.yy HH:mm", "d MMM yyyy, HH:mm", "d MMMM yyyy à HH:mm", "EEEE, d MMMM yyyy à HH:mm"],
    months: [["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"], ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."]],
    weekdays: ["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"], ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."]],
    standalone_weekdays: [["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"], ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_21: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 2,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}mil", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}M", forms: &[] },
        Compact { magnitude: 10, scale: 9, other: "#\u{a0}mil\u{a0}M", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}B", forms: &[] }
    ],
};

static DATES_22: Dates = Dates {
    date: ["d/M/yy", "d MMM yyyy", "d' de 'MMMM' de 'yyyy", "EEEE, d' de 'MMMM' de 'yyyy"],
    time: ["H:mm", "H:mm:ss", "H:mm:ss' UTC'", "H:mm:ss' (tiempo universal coordinado)'"],
    datetime: ["d/M/yy, H:mm", "d MMM yyyy, H:mm", "d' de 'MMMM' de 'yyyy' a las 'H:mm", "EEEE, d' de 'MMMM' de 'yyyy, H:mm"],
    months: [["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"], ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic"]],
    weekdays: ["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"], ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic"]],
    standalone_weekdays: [["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"], ["dom", "lun", "mar", "mié", "jue", "vie", "sáb"]],
    standalone_periods: ["a.\u{202f}m.", "a.\u{202f}m.", "a.\u{202f}m.", "a.\u{202f}m.", "a.\u{202f}m.", "a.\u{202f}m.", "a.\u{202f}m.", "a.\u{202f}m.", "a.\u{202f}m.", "a.\u{202f}m.", "a.\u{202f}m.", "a.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m.", "p.\u{202f}m."],
};

static NUMBERS_23: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("MXN", "$")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}k", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}M", forms: &[] },
        Compact { magnitude: 10, scale: 9, other: "#\u{a0}mil\u{a0}M", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}B", forms: &[] }
    ],
};

static DATES_24: Dates = Dates {
    date: ["dd/MM/yy", "d MMM yyyy", "d' de 'MMMM' de 'yyyy", "EEEE, d' de 'MMMM' de 'yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a' hora universal coordinada'"],
    datetime: ["dd/MM/yy, h:mm a", "d MMM yyyy, h:mm a", "d' de 'MMMM' de 'yyyy' a las 'h:mm a", "EEEE, d' de 'MMMM' de 'yyyy, h:mm a"],
    months: [["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"], ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic"]],
    weekdays: ["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"],
    periods: ["a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m."],
    standalone_months: [["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"], ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic"]],
    standalone_weekdays: [["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"], ["dom", "lun", "mar", "mié", "jue", "vie", "sáb"]],
    standalone_periods: ["a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m."],
};

static NUMBERS_25: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}K", forms: &[] },
        Compact { magnitude: 4, scale: 3, other: "#\u{a0}k", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}M", forms: &[] },
        Compact { magnitude: 10, scale: 9, other: "#\u{a0}mil\u{a0}M", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}B", forms: &[] }
    ],
};

static DATES_26: Dates = Dates {
    date: ["d/M/yy", "d MMM yyyy", "d' de 'MMMM' de 'yyyy", "EEEE, d' de 'MMMM' de 'yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a' hora universal coordinada'"],
    datetime: ["d/M/yy, h:mm a", "d MMM yyyy, h:mm a", "d' de 'MMMM' de 'yyyy' a las 'h:mm a", "EEEE, d' de 'MMMM' de 'yyyy, h:mm a"],
    months: [["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"], ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic"]],
    weekdays: ["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"],
    periods: ["a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m."],
    standalone_months: [["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"], ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic"]],
    standalone_weekdays: [["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"], ["dom", "lun", "mar", "mié", "jue", "vie", "sáb"]],
    standalone_periods: ["a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m."],
};

static NUMBERS_27: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 2,
    percent: "#%",
    percent_negative: "-#%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("EUR", "€"), ("GBP", "£"), ("CNY", "CN¥"), ("CAD", "CA$"), ("AUD", "A$"), ("ILS", "₪"), ("PHP", "₱")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#K", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}Mln", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}Mld", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}Bln", forms: &[] }
    ],
};

static DATES_28: Dates = Dates {
    date: ["dd/MM/yy", "d MMM yyyy", "d MMMM yyyy", "EEEE d MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' Tempo coordinato universale'"],
    datetime: ["dd/MM/yy, HH:mm", "d MMM yyyy, HH:mm", "d MMMM yyyy' alle ore 'HH:mm", "EEEE d MMMM yyyy' alle ore 'HH:mm"],
    months: [["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"], ["gen", "feb", "mar", "apr", "mag", "giu", "lug", "ago", "set", "ott", "nov", "dic"]],
    weekdays: ["domenica", "lunedì", "martedì", "mercoledì", "giovedì", "venerdì", "sabato"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"], ["gen", "feb", "mar", "apr", "mag", "giu", "lug", "ago", "set", "ott", "nov", "dic"]],
    standalone_weekdays: [["domenica", "lunedì", "martedì", "mercoledì", "giovedì", "venerdì", "sabato"], ["dom", "lun", "mar", "mer", "gio", "ven", "sab"]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_29: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤\u{a0}#",
    currency_negative: "-¤\u{a0}#",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}mil", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}mi", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}bi", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}tri", forms: &[] }
    ],
};

static DATES_30: Dates = Dates {
    date: ["dd/MM/yyyy", "d' de 'MMM' de 'yyyy", "d' de 'MMMM' de 'yyyy", "EEEE, d' de 'MMMM' de 'yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' Horário Universal Coordenado'"],
    datetime: ["dd/MM/yyyy, HH:mm", "d' de 'MMM' de 'yyyy, HH:mm", "d' de 'MMMM' de 'yyyy' às 'HH:mm", "EEEE, d' de 'MMMM' de 'yyyy' às 'HH:mm"],
    months: [["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"], ["jan.", "fev.", "mar.", "abr.", "mai.", "jun.", "jul.", "ago.", "set.", "out.", "nov.", "dez."]],
    weekdays: ["domingo", "segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"], ["jan.", "fev.", "mar.", "abr.", "mai.", "jun.", "jul.", "ago.", "set.", "out.", "nov.", "dez."]],
    standalone_weekdays: [["domingo", "segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado"], ["dom.", "seg.", "ter.", "qua.", "qui.", "sex.", "sáb."]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_31: Numbers = Numbers {
    zero: '0',
    group: "\u{a0}",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 2,
    percent: "#%",
    percent_negative: "-#%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{a0}",
    currency_decimal: ",",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}mil", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}M", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}mM", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}Bi", forms: &[] }
    ],
};

static DATES_32: Dates = Dates {
    date: ["dd/MM/yy", "dd/MM/yyyy", "d' de 'MMMM' de 'yyyy", "EEEE, d' de 'MMMM' de 'yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' Hora Coordenada Universal'"],
    datetime: ["dd/MM/yy, HH:mm", "dd/MM/yyyy, HH:mm", "d' de 'MMMM' de 'yyyy' às 'HH:mm", "EEEE, d' de 'MMMM' de 'yyyy' às 'HH:mm"],
    months: [["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"], ["jan.", "fev.", "mar.", "abr.", "mai.", "jun.", "jul.", "ago.", "set.", "out.", "nov.", "dez."]],
    weekdays: ["domingo", "segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"], ["jan.", "fev.", "mar.", "abr.", "mai.", "jun.", "jul.", "ago.", "set.", "out.", "nov.", "dez."]],
    standalone_weekdays: [["domingo", "segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado"], ["domingo", "segunda", "terça", "quarta", "quinta", "sexta", "sábado"]],
    standalone_periods: ["a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m."],
};

static NUMBERS_33: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤\u{a0}#",
    currency_negative: "¤\u{a0}-#",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "C$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#K", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}mln.", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}mld.", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}bln.", forms: &[] }
    ],
};

static DATES_34: Dates = Dates {
    date: ["dd-MM-yyyy", "d MMM yyyy", "d MMMM yyyy", "EEEE d MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' gecoördineerde wereldtijd'"],
    datetime: ["dd-MM-yyyy, HH:mm", "d MMM yyyy, HH:mm", "d MMMM yyyy' om 'HH:mm", "EEEE d MMMM yyyy' om 'HH:mm"],
    months: [["januari", "februari", "maart", "april", "mei", "juni", "juli", "augustus", "september", "oktober", "november", "december"], ["jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec"]],
    weekdays: ["zondag", "maandag", "dinsdag", "woensdag", "donderdag", "vrijdag", "zaterdag"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["januari", "februari", "maart", "april", "mei", "juni", "juli", "augustus", "september", "oktober", "november", "december"], ["jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec"]],
    standalone_weekdays: [["zondag", "maandag", "dinsdag", "woensdag", "donderdag", "vrijdag", "zaterdag"], ["zo", "ma", "di", "wo", "do", "vr", "za"]],
    standalone_periods: ["a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m."],
};

static NUMBERS_35: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "\u{200e}-",
    indian: false,
    min_grouping: 1,
    percent: "#\u{200e}%\u{200e}",
    percent_negative: "\u{200e}-#\u{200e}%\u{200e}",
    currency: "\u{200f}#\u{a0}¤",
    currency_negative: "\u{200f}\u{200e}-#\u{a0}¤",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "UK£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫"), ("SAR", "ر.س.\u{200f}"), ("AED", "د.إ.\u{200f}"), ("EGP", "ج.م.\u{200f}"), ("KWD", "د.ك.\u{200f}"), ("QAR", "ر.ق.\u{200f}"), ("MAD", "د.م.\u{200f}")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}ألف", forms: &[("few", "#\u{a0}آلاف")] },
        Compact { magnitude: 4, scale: 3, other: "#\u{a0}ألف", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}مليون", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}مليار", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}ترليون", forms: &[] }
    ],
};

static DATES_36: Dates = Dates {
    date: ["d\u{200f}/M\u{200f}/yyyy", "dd\u{200f}/MM\u{200f}/yyyy", "d MMMM yyyy", "EEEE، d MMMM yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a التوقيت العالمي المنسق"],
    datetime: ["d\u{200f}/M\u{200f}/yyyy، h:mm a", "dd\u{200f}/MM\u{200f}/yyyy، h:mm a", "d MMMM yyyy في h:mm a", "EEEE، d MMMM yyyy في h:mm a"],
    months: [["يناير", "فبراير", "مارس", "أبريل", "مايو", "يونيو", "يوليو", "أغسطس", "سبتمبر", "أكتوبر", "نوفمبر", "ديسمبر"], ["يناير", "فبراير", "مارس", "أبريل", "مايو", "يونيو", "يوليو", "أغسطس", "سبتمبر", "أكتوبر", "نوفمبر", "ديسمبر"]],
    weekdays: ["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"],
    periods: ["ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م"],
    standalone_months: [["يناير", "فبراير", "مارس", "أبريل", "مايو", "يونيو", "يوليو", "أغسطس", "سبتمبر", "أكتوبر", "نوفمبر", "ديسمبر"], ["يناير", "فبراير", "مارس", "أبريل", "مايو", "يونيو", "يوليو", "أغسطس", "سبتمبر", "أكتوبر", "نوفمبر", "ديسمبر"]],
    standalone_weekdays: [["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"], ["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"]],
    standalone_periods: ["ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م"],
};

static NUMBERS_37: Numbers = Numbers {
    zero: '٠',
    group: "٬",
    decimal: "٫",
    minus: "-",
    negative: "\u{61c}-",
    indian: false,
    min_grouping: 1,
    percent: "#٪\u{61c}",
    percent_negative: "\u{61c}-#٪\u{61c}",
    currency: "\u{200f}#\u{a0}¤",
    currency_negative: "\u{61c}-\u{200f}#\u{a0}¤",
    currency_group: "٬",
    currency_decimal: "٫",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "UK£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫"), ("SAR", "ر.س.\u{200f}"), ("AED", "د.إ.\u{200f}"), ("EGP", "ج.م.\u{200f}"), ("KWD", "د.ك.\u{200f}"), ("QAR", "ر.ق.\u{200f}"), ("MAD", "د.م.\u{200f}")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}ألف", forms: &[("few", "#\u{a0}آلاف")] },
        Compact { magnitude: 4, scale: 3, other: "#\u{a0}ألف", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}مليون", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}مليار", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}ترليون", forms: &[] }
    ],
};

static NUMBERS_38: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "\u{200e}-",
    indian: false,
    min_grouping: 1,
    percent: "#\u{200e}%\u{200e}",
    percent_negative: "\u{200e}-#\u{200e}%\u{200e}",
    currency: "\u{200f}#\u{a0}¤",
    currency_negative: "\u{200f}\u{200e}-#\u{a0}¤",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "UK£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫"), ("SAR", "ر.س.\u{200f}"), ("AED", "د.إ.\u{200f}"), ("EGP", "ج.م.\u{200f}"), ("KWD", "د.ك.\u{200f}"), ("QAR", "ر.ق.\u{200f}"), ("MAD", "د.م.\u{200f}")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}ألف", forms: &[("few", "#\u{a0}آلاف")] },
        Compact { magnitude: 4, scale: 3, other: "#\u{a0}ألف", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}مليون", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}مليار", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}ترليون", forms: &[] }
    ],
};

static DATES_39: Dates = Dates {
    date: ["d\u{200f}/M\u{200f}/yyyy", "dd\u{200f}/MM\u{200f}/yyyy", "d MMMM yyyy", "EEEE، d MMMM yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a التوقيت العالمي المنسق"],
    datetime: ["d\u{200f}/M\u{200f}/yyyy، h:mm a", "dd\u{200f}/MM\u{200f}/yyyy، h:mm a", "d MMMM yyyy في h:mm a", "EEEE، d MMMM yyyy في h:mm a"],
    months: [["جانفي", "فيفري", "مارس", "أفريل", "ماي", "جوان", "جويلية", "أوت", "سبتمبر", "أكتوبر", "نوفمبر", "ديسمبر"], ["جانفي", "فيفري", "مارس", "أفريل", "ماي", "جوان", "جويلية", "أوت", "سبتمبر", "أكتوبر", "نوفمبر", "ديسمبر"]],
    weekdays: ["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"],
    periods: ["ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م"],
    standalone_months: [["جانفي", "فيفري", "مارس", "أفريل", "ماي", "جوان", "جويلية", "أوت", "سبتمبر", "أكتوبر", "نوفمبر", "ديسمبر"], ["جانفي", "فيفري", "مارس", "أفريل", "ماي", "جوان", "جويلية", "أوت", "سبتمبر", "أكتوبر", "نوفمبر", "ديسمبر"]],
    standalone_weekdays: [["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"], ["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"]],
    standalone_periods: ["ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م"],
};

static DATES_40: Dates = Dates {
    date: ["d\u{200f}/M\u{200f}/yyyy", "dd\u{200f}/MM\u{200f}/yyyy", "d MMMM yyyy", "EEEE، d MMMM yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a التوقيت العالمي المنسق"],
    datetime: ["d\u{200f}/M\u{200f}/yyyy، h:mm a", "dd\u{200f}/MM\u{200f}/yyyy، h:mm a", "d MMMM yyyy في h:mm a", "EEEE، d MMMM yyyy في h:mm a"],
    months: [["كانون الثاني", "شباط", "آذار", "نيسان", "أيار", "حزيران", "تموز", "آب", "أيلول", "تشرين الأول", "تشرين الثاني", "كانون الأول"], ["كانون الثاني", "شباط", "آذار", "نيسان", "أيار", "حزيران", "تموز", "آب", "أيلول", "تشرين الأول", "تشرين الثاني", "كانون الأول"]],
    weekdays: ["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"],
    periods: ["ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م"],
    standalone_months: [["كانون الثاني", "شباط", "آذار", "نيسان", "أيار", "حزيران", "تموز", "آب", "أيلول", "تشرين الأول", "تشرين الثاني", "كانون الأول"], ["كانون الثاني", "شباط", "آذار", "نيسان", "أيار", "حزيران", "تموز", "آب", "أيلول", "تشرين الأول", "تشرين الثاني", "كانون الأول"]],
    standalone_weekdays: [["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"], ["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"]],
    standalone_periods: ["ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م"],
};

static DATES_41: Dates = Dates {
    date: ["d\u{200f}/M\u{200f}/yyyy", "dd\u{200f}/MM\u{200f}/yyyy", "d MMMM yyyy", "EEEE، d MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss التوقيت العالمي المنسق"],
    datetime: ["d\u{200f}/M\u{200f}/yyyy، HH:mm", "dd\u{200f}/MM\u{200f}/yyyy، HH:mm", "d MMMM yyyy في HH:mm", "EEEE، d MMMM yyyy في HH:mm"],
    months: [["يناير", "فبراير", "مارس", "أبريل", "ماي", "يونيو", "يوليوز", "غشت", "شتنبر", "أكتوبر", "نونبر", "دجنبر"], ["يناير", "فبراير", "مارس", "أبريل", "ماي", "يونيو", "يوليوز", "غشت", "شتنبر", "أكتوبر", "نونبر", "دجنبر"]],
    weekdays: ["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["يناير", "فبراير", "مارس", "أبريل", "ماي", "يونيو", "يوليوز", "غشت", "شتنبر", "أكتوبر", "نونبر", "دجنبر"], ["يناير", "فبراير", "مارس", "أبريل", "ماي", "يونيو", "يوليوز", "غشت", "شتنبر", "أكتوبر", "نونبر", "دجنبر"]],
    standalone_weekdays: [["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"], ["الأحد", "الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت"]],
    standalone_periods: ["ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "ص", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م", "م"],
};

static NUMBERS_42: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "￥"), ("CNY", "元"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "A$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 4, scale: 4, other: "#万", forms: &[] },
        Compact { magnitude: 8, scale: 8, other: "#億", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#兆", forms: &[] },
        Compact { magnitude: 16, scale: 16, other: "#京", forms: &[] }
    ],
};

static DATES_43: Dates = Dates {
    date: ["yyyy/MM/dd", "yyyy/MM/dd", "yyyy年M月d日", "yyyy年M月d日EEEE"],
    time: ["H:mm", "H:mm:ss", "H:mm:ss' UTC'", "H時mm分ss秒 協定世界時"],
    datetime: ["yyyy/MM/dd H:mm", "yyyy/MM/dd H:mm", "yyyy年M月d日 H:mm", "yyyy年M月d日EEEE H:mm"],
    months: [["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"], ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"]],
    weekdays: ["日曜日", "月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"], ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"]],
    standalone_weekdays: [["日曜日", "月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日"], ["日", "月", "火", "水", "木", "金", "土"]],
    standalone_periods: ["午前", "午前", "午前", "午前", "午前", "午前", "午前", "午前", "午前", "午前", "午前", "午前", "午後", "午後", "午後", "午後", "午後", "午後", "午後", "午後", "午後", "午後", "午後", "午後"],
};

static NUMBERS_44: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 4, scale: 4, other: "#万", forms: &[] },
        Compact { magnitude: 8, scale: 8, other: "#亿", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#万亿", forms: &[] }
    ],
};

static DATES_45: Dates = Dates {
    date: ["yyyy/M/d", "yyyy年M月d日", "yyyy年M月d日", "yyyy年M月d日EEEE"],
    time: ["HH:mm", "HH:mm:ss", "'UTC 'HH:mm:ss", "协调世界时 HH:mm:ss"],
    datetime: ["yyyy/M/d HH:mm", "yyyy年M月d日 HH:mm", "yyyy年M月d日 HH:mm", "yyyy年M月d日EEEE HH:mm"],
    months: [["一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"], ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"]],
    weekdays: ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"], ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"]],
    standalone_weekdays: [["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"], ["周日", "周一", "周二", "周三", "周四", "周五", "周六"]],
    standalone_periods: ["上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午"],
};

static NUMBERS_46: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "￦"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 4, scale: 4, other: "#萬", forms: &[] },
        Compact { magnitude: 8, scale: 8, other: "#億", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#兆", forms: &[] }
    ],
};

static DATES_47: Dates = Dates {
    date: ["yyyy/M/d", "yyyy年M月d日", "yyyy年M月d日", "yyyy年M月d日 EEEE"],
    time: ["ah:mm", "ah:mm:ss", "ah:mm:ss' [UTC]'", "ah:mm:ss [世界標準時間]"],
    datetime: ["yyyy/M/d\u{2009}ah:mm", "yyyy年M月d日 ah:mm", "yyyy年M月d日 ah:mm", "yyyy年M月d日 EEEE ah:mm"],
    months: [["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"], ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"]],
    weekdays: ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"],
    periods: ["凌晨", "凌晨", "凌晨", "凌晨", "凌晨", "清晨", "清晨", "清晨", "上午", "上午", "上午", "上午", "中午", "下午", "下午", "下午", "下午", "下午", "下午", "晚上", "晚上", "晚上", "晚上", "晚上"],
    standalone_months: [["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"], ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"]],
    standalone_weekdays: [["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"], ["週日", "週一", "週二", "週三", "週四", "週五", "週六"]],
    standalone_periods: ["上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午"],
};

static NUMBERS_48: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#K", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#M", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#B", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#T", forms: &[] }
    ],
};

static DATES_49: Dates = Dates {
    date: ["d/M/yyyy", "yyyy年M月d日", "yyyy年M月d日", "yyyy年M月d日EEEE"],
    time: ["ah:mm", "ah:mm:ss", "ah:mm:ss' [UTC]'", "ah:mm:ss [世界標準時間]"],
    datetime: ["d/M/yyyy\u{2009}ah:mm", "yyyy年M月d日 ah:mm", "yyyy年M月d日 ah:mm", "yyyy年M月d日EEEE ah:mm"],
    months: [["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"], ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"]],
    weekdays: ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"],
    periods: ["上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午"],
    standalone_months: [["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"], ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"]],
    standalone_weekdays: [["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"], ["週日", "週一", "週二", "週三", "週四", "週五", "週六"]],
    standalone_periods: ["上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "上午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午", "下午"],
};

static NUMBERS_50: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#천", forms: &[] },
        Compact { magnitude: 4, scale: 4, other: "#만", forms: &[] },
        Compact { magnitude: 8, scale: 8, other: "#억", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#조", forms: &[] }
    ],
};

static DATES_51: Dates = Dates {
    date: ["yy. M. d.", "yyyy. M. d.", "yyyy년 MMMM d일", "yyyy년 MMMM d일 EEEE"],
    time: ["a h:mm", "a h:mm:ss", "a h시 m분 s'초 UTC'", "a h시 m분 s초 협정 세계시"],
    datetime: ["yy. M. d. a h:mm", "yyyy. M. d. a h:mm", "yyyy년 MMMM d일 a h:mm", "yyyy년 MMMM d일 EEEE a h:mm"],
    months: [["1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월"], ["1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월"]],
    weekdays: ["일요일", "월요일", "화요일", "수요일", "목요일", "금요일", "토요일"],
    periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
    standalone_months: [["1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월"], ["1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월"]],
    standalone_weekdays: [["일요일", "월요일", "화요일", "수요일", "목요일", "금요일", "토요일"], ["일", "월", "화", "수", "목", "금", "토"]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_52: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "-",
    indian: true,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "A$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}हज़ार", forms: &[] },
        Compact { magnitude: 5, scale: 5, other: "#\u{a0}लाख", forms: &[] },
        Compact { magnitude: 7, scale: 7, other: "#\u{a0}क॰", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}अ॰", forms: &[] },
        Compact { magnitude: 11, scale: 11, other: "#\u{a0}ख॰", forms: &[] },
        Compact { magnitude: 13, scale: 13, other: "#\u{a0}नील", forms: &[] }
    ],
};

static DATES_53: Dates = Dates {
    date: ["d/M/yy", "d MMM yyyy", "d MMMM yyyy", "EEEE, d MMMM yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a समन्वित वैश्विक समय"],
    datetime: ["d/M/yy, h:mm a", "d MMM yyyy, h:mm a", "d MMMM yyyy को h:mm a बजे", "EEEE, d MMMM yyyy को h:mm a बजे"],
    months: [["जनवरी", "फ़रवरी", "मार्च", "अप्रैल", "मई", "जून", "जुलाई", "अगस्त", "सितंबर", "अक्टूबर", "नवंबर", "दिसंबर"], ["जन॰", "फ़र॰", "मार्च", "अप्रैल", "मई", "जून", "जुल॰", "अग॰", "सित॰", "अक्टू॰", "नव॰", "दिस॰"]],
    weekdays: ["रविवार", "सोमवार", "मंगलवार", "बुधवार", "गुरुवार", "शुक्रवार", "शनिवार"],
    periods: ["am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm"],
    standalone_months: [["जनवरी", "फ़रवरी", "मार्च", "अप्रैल", "मई", "जून", "जुलाई", "अगस्त", "सितंबर", "अक्टूबर", "नवंबर", "दिसंबर"], ["जन॰", "फ़र॰", "मार्च", "अप्रैल", "मई", "जून", "जुल॰", "अग॰", "सित॰", "अक्टू॰", "नव॰", "दिस॰"]],
    standalone_weekdays: [["रविवार", "सोमवार", "मंगलवार", "बुधवार", "गुरुवार", "शुक्रवार", "शनिवार"], ["रवि", "सोम", "मंगल", "बुध", "गुरु", "शुक्र", "शनि"]],
    standalone_periods: ["am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "am", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm", "pm"],
};

static NUMBERS_54: Numbers = Numbers {
    zero: '0',
    group: "\u{a0}",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{a0}",
    currency_decimal: ",",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "A$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫"), ("RUB", "₽")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}тыс.", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}млн", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}млрд", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}трлн", forms: &[] }
    ],
};

static DATES_55: Dates = Dates {
    date: ["dd.MM.yyyy", "d MMM yyyy г.", "d MMMM yyyy г.", "EEEE, d MMMM yyyy г."],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss Всемирное координированное время"],
    datetime: ["dd.MM.yyyy, HH:mm", "d MMM yyyy г., HH:mm", "d MMMM yyyy г. в HH:mm", "EEEE, d MMMM yyyy г. в HH:mm"],
    months: [["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"], ["янв.", "февр.", "мар.", "апр.", "мая", "июн.", "июл.", "авг.", "сент.", "окт.", "нояб.", "дек."]],
    weekdays: ["воскресенье", "понедельник", "вторник", "среда", "четверг", "пятница", "суббота"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["январь", "февраль", "март", "апрель", "май", "июнь", "июль", "август", "сентябрь", "октябрь", "ноябрь", "декабрь"], ["янв.", "февр.", "март", "апр.", "май", "июнь", "июль", "авг.", "сент.", "окт.", "нояб.", "дек."]],
    standalone_weekdays: [["воскресенье", "понедельник", "вторник", "среда", "четверг", "пятница", "суббота"], ["вс", "пн", "вт", "ср", "чт", "пт", "сб"]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_56: Numbers = Numbers {
    zero: '0',
    group: "\u{a0}",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{a0}",
    currency_decimal: ",",
    symbols: &[("JPY", "¥")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}тис.", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}млн", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}млрд", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}трлн", forms: &[] }
    ],
};

static DATES_57: Dates = Dates {
    date: ["dd.MM.yy", "d MMM yyyy р.", "d MMMM yyyy р.", "EEEE, d MMMM yyyy р."],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss за всесвітнім координованим часом"],
    datetime: ["dd.MM.yy, HH:mm", "d MMM yyyy р., HH:mm", "d MMMM yyyy р. о HH:mm", "EEEE, d MMMM yyyy р. о HH:mm"],
    months: [["січня", "лютого", "березня", "квітня", "травня", "червня", "липня", "серпня", "вересня", "жовтня", "листопада", "грудня"], ["січ.", "лют.", "бер.", "квіт.", "трав.", "черв.", "лип.", "серп.", "вер.", "жовт.", "лист.", "груд."]],
    weekdays: ["неділя", "понеділок", "вівторок", "середа", "четвер", "пʼятниця", "субота"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["січень", "лютий", "березень", "квітень", "травень", "червень", "липень", "серпень", "вересень", "жовтень", "листопад", "грудень"], ["січ.", "лют.", "бер.", "квіт.", "трав.", "черв.", "лип.", "серп.", "вер.", "жовт.", "лист.", "груд."]],
    standalone_weekdays: [["неділя", "понеділок", "вівторок", "середа", "четвер", "пʼятниця", "субота"], ["нд", "пн", "вт", "ср", "чт", "пт", "сб"]],
    standalone_periods: ["дп", "дп", "дп", "дп", "дп", "дп", "дп", "дп", "дп", "дп", "дп", "дп", "пп", "пп", "пп", "пп", "пп", "пп", "пп", "пп", "пп", "пп", "пп", "пп"],
};

static NUMBERS_58: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "%#",
    percent_negative: "-%#",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫"), ("TRY", "₺")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}B", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}Mn", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}Mr", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}Tn", forms: &[] }
    ],
};

static DATES_59: Dates = Dates {
    date: ["d.MM.yyyy", "d MMM yyyy", "d MMMM yyyy", "d MMMM yyyy EEEE"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' Eş Güdümlü Evrensel Zaman'"],
    datetime: ["d.MM.yyyy HH:mm", "d MMM yyyy HH:mm", "d MMMM yyyy HH:mm", "d MMMM yyyy EEEE HH:mm"],
    months: [["Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"], ["Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu", "Eyl", "Eki", "Kas", "Ara"]],
    weekdays: ["Pazar", "Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma", "Cumartesi"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"], ["Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu", "Eyl", "Eki", "Kas", "Ara"]],
    standalone_weekdays: [["Pazar", "Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma", "Cumartesi"], ["Paz", "Pzt", "Sal", "Çar", "Per", "Cum", "Cmt"]],
    standalone_periods: ["ÖÖ", "ÖÖ", "ÖÖ", "ÖÖ", "ÖÖ", "ÖÖ", "ÖÖ", "ÖÖ", "ÖÖ", "ÖÖ", "ÖÖ", "ÖÖ", "ÖS", "ÖS", "ÖS", "ÖS", "ÖS", "ÖS", "ÖS", "ÖS", "ÖS", "ÖS", "ÖS", "ÖS"],
};

static NUMBERS_60: Numbers = Numbers {
    zero: '0',
    group: "\u{a0}",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 2,
    percent: "#%",
    percent_negative: "-#%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{a0}",
    currency_decimal: ",",
    symbols: &[("EUR", "€"), ("BRL", "R$")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}tys.", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}mln", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}mld", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}bln", forms: &[] }
    ],
};

static DATES_61: Dates = Dates {
    date: ["d.MM.yyyy", "d MMM yyyy", "d MMMM yyyy", "EEEE, d MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' uniwersalny czas koordynowany'"],
    datetime: ["d.MM.yyyy, HH:mm", "d MMM yyyy, HH:mm", "d MMMM yyyy HH:mm", "EEEE, d MMMM yyyy HH:mm"],
    months: [["stycznia", "lutego", "marca", "kwietnia", "maja", "czerwca", "lipca", "sierpnia", "września", "października", "listopada", "grudnia"], ["sty", "lut", "mar", "kwi", "maj", "cze", "lip", "sie", "wrz", "paź", "lis", "gru"]],
    weekdays: ["niedziela", "poniedziałek", "wtorek", "środa", "czwartek", "piątek", "sobota"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["styczeń", "luty", "marzec", "kwiecień", "maj", "czerwiec", "lipiec", "sierpień", "wrzesień", "październik", "listopad", "grudzień"], ["sty", "lut", "mar", "kwi", "maj", "cze", "lip", "sie", "wrz", "paź", "lis", "gru"]],
    standalone_weekdays: [["niedziela", "poniedziałek", "wtorek", "środa", "czwartek", "piątek", "sobota"], ["niedz.", "pon.", "wt.", "śr.", "czw.", "pt.", "sob."]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_62: Numbers = Numbers {
    zero: '0',
    group: "\u{a0}",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{a0}",
    currency_decimal: ",",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}tis.", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}mil.", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}mld.", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}bil.", forms: &[] }
    ],
};

static DATES_63: Dates = Dates {
    date: ["dd.MM.yy", "d. M. yyyy", "d. MMMM yyyy", "EEEE d. MMMM yyyy"],
    time: ["H:mm", "H:mm:ss", "H:mm:ss' UTC'", "H:mm:ss', koordinovaný světový čas'"],
    datetime: ["dd.MM.yy H:mm", "d. M. yyyy H:mm", "d. MMMM yyyy' v 'H:mm", "EEEE d. MMMM yyyy' v 'H:mm"],
    months: [["ledna", "února", "března", "dubna", "května", "června", "července", "srpna", "září", "října", "listopadu", "prosince"], ["led", "úno", "bře", "dub", "kvě", "čvn", "čvc", "srp", "zář", "říj", "lis", "pro"]],
    weekdays: ["neděle", "pondělí", "úterý", "středa", "čtvrtek", "pátek", "sobota"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["leden", "únor", "březen", "duben", "květen", "červen", "červenec", "srpen", "září", "říjen", "listopad", "prosinec"], ["led", "úno", "bře", "dub", "kvě", "čvn", "čvc", "srp", "zář", "říj", "lis", "pro"]],
    standalone_weekdays: [["neděle", "pondělí", "úterý", "středa", "čtvrtek", "pátek", "sobota"], ["ne", "po", "út", "st", "čt", "pá", "so"]],
    standalone_periods: ["dop.", "dop.", "dop.", "dop.", "dop.", "dop.", "dop.", "dop.", "dop.", "dop.", "dop.", "dop.", "odp.", "odp.", "odp.", "odp.", "odp.", "odp.", "odp.", "odp.", "odp.", "odp.", "odp.", "odp."],
};

static NUMBERS_64: Numbers = Numbers {
    zero: '0',
    group: "\u{a0}",
    decimal: ",",
    minus: "−",
    negative: "−",
    indian: false,
    min_grouping: 1,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{a0}",
    currency_decimal: ",",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("CAD", "CA$"), ("BRL", "BR$"), ("MXN", "MX$"), ("ILS", "₪"), ("EGP", "EG£")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}tn", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}mn", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}md", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}bn", forms: &[] }
    ],
};

static DATES_65: Dates = Dates {
    date: ["yyyy-MM-dd", "d MMM yyyy", "d MMMM yyyy", "EEEE d MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' koordinerad universell tid'"],
    datetime: ["yyyy-MM-dd HH:mm", "d MMM yyyy HH:mm", "d MMMM yyyy' kl. 'HH:mm", "EEEE d MMMM yyyy' kl. 'HH:mm"],
    months: [["januari", "februari", "mars", "april", "maj", "juni", "juli", "augusti", "september", "oktober", "november", "december"], ["jan.", "feb.", "mars", "apr.", "maj", "juni", "juli", "aug.", "sep.", "okt.", "nov.", "dec."]],
    weekdays: ["söndag", "måndag", "tisdag", "onsdag", "torsdag", "fredag", "lördag"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["januari", "februari", "mars", "april", "maj", "juni", "juli", "augusti", "september", "oktober", "november", "december"], ["jan.", "feb.", "mars", "apr.", "maj", "juni", "juli", "aug.", "sep.", "okt.", "nov.", "dec."]],
    standalone_weekdays: [["söndag", "måndag", "tisdag", "onsdag", "torsdag", "fredag", "lördag"], ["sön", "mån", "tis", "ons", "tors", "fre", "lör"]],
    standalone_periods: ["fm", "fm", "fm", "fm", "fm", "fm", "fm", "fm", "fm", "fm", "fm", "fm", "em", "em", "em", "em", "em", "em", "em", "em", "em", "em", "em", "em"],
};

static NUMBERS_66: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}t", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}mio.", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}mia.", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}bio.", forms: &[] }
    ],
};

static DATES_67: Dates = Dates {
    date: ["dd.MM.yyyy", "d. MMM yyyy", "d. MMMM yyyy", "EEEE' den 'd. MMMM yyyy"],
    time: ["HH.mm", "HH.mm.ss", "HH.mm.ss' UTC'", "HH.mm.ss' Koordineret universaltid'"],
    datetime: ["dd.MM.yyyy, HH.mm", "d. MMM yyyy, HH.mm", "d. MMMM yyyy' kl. 'HH.mm", "EEEE' den 'd. MMMM yyyy' kl. 'HH.mm"],
    months: [["januar", "februar", "marts", "april", "maj", "juni", "juli", "august", "september", "oktober", "november", "december"], ["jan.", "feb.", "mar.", "apr.", "maj", "jun.", "jul.", "aug.", "sep.", "okt.", "nov.", "dec."]],
    weekdays: ["søndag", "mandag", "tirsdag", "onsdag", "torsdag", "fredag", "lørdag"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["januar", "februar", "marts", "april", "maj", "juni", "juli", "august", "september", "oktober", "november", "december"], ["jan.", "feb.", "mar.", "apr.", "maj", "jun.", "jul.", "aug.", "sep.", "okt.", "nov.", "dec."]],
    standalone_weekdays: [["søndag", "mandag", "tirsdag", "onsdag", "torsdag", "fredag", "lørdag"], ["søn.", "man.", "tirs.", "ons.", "tors.", "fre.", "lør."]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_68: Numbers = Numbers {
    zero: '0',
    group: "\u{a0}",
    decimal: ",",
    minus: "−",
    negative: "−",
    indian: false,
    min_grouping: 1,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{a0}",
    currency_decimal: ",",
    symbols: &[("EUR", "€"), ("GBP", "£")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#k", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}mill.", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}mrd.", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}bill.", forms: &[] }
    ],
};

static DATES_69: Dates = Dates {
    date: ["dd.MM.yyyy", "d. MMM yyyy", "d. MMMM yyyy", "EEEE d. MMMM yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' koordinert universaltid'"],
    datetime: ["dd.MM.yyyy, HH:mm", "d. MMM yyyy, HH:mm", "d. MMMM yyyy' kl. 'HH:mm", "EEEE d. MMMM yyyy' kl. 'HH:mm"],
    months: [["januar", "februar", "mars", "april", "mai", "juni", "juli", "august", "september", "oktober", "november", "desember"], ["jan.", "feb.", "mars", "apr.", "mai", "juni", "juli", "aug.", "sep.", "okt.", "nov.", "des."]],
    weekdays: ["søndag", "mandag", "tirsdag", "onsdag", "torsdag", "fredag", "lørdag"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["januar", "februar", "mars", "april", "mai", "juni", "juli", "august", "september", "oktober", "november", "desember"], ["jan", "feb", "mar", "apr", "mai", "jun", "jul", "aug", "sep", "okt", "nov", "des"]],
    standalone_weekdays: [["søndag", "mandag", "tirsdag", "onsdag", "torsdag", "fredag", "lørdag"], ["søn.", "man.", "tir.", "ons.", "tor.", "fre.", "lør."]],
    standalone_periods: ["a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "a.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m.", "p.m."],
};

static NUMBERS_70: Numbers = Numbers {
    zero: '0',
    group: "\u{a0}",
    decimal: ",",
    minus: "−",
    negative: "−",
    indian: false,
    min_grouping: 1,
    percent: "#\u{a0}%",
    percent_negative: "-#\u{a0}%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: "\u{a0}",
    currency_decimal: ",",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}t.", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}milj.", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}mrd.", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}bilj.", forms: &[] }
    ],
};

static DATES_71: Dates = Dates {
    date: ["d.M.yyyy", "d.M.yyyy", "d. MMMM yyyy", "EEEE d. MMMM yyyy"],
    time: ["H.mm", "H.mm.ss", "H.mm.ss' UTC'", "H.mm.ss' UTC-yleisaika'"],
    datetime: ["d.M.yyyy' klo 'H.mm", "d.M.yyyy' klo 'H.mm", "d. MMMM yyyy' klo 'H.mm", "EEEE d. MMMM yyyy' klo 'H.mm"],
    months: [["tammikuuta", "helmikuuta", "maaliskuuta", "huhtikuuta", "toukokuuta", "kesäkuuta", "heinäkuuta", "elokuuta", "syyskuuta", "lokakuuta", "marraskuuta", "joulukuuta"], ["tammi", "helmi", "maalis", "huhti", "touko", "kesä", "heinä", "elo", "syys", "loka", "marras", "joulu"]],
    weekdays: ["sunnuntai", "maanantai", "tiistai", "keskiviikko", "torstai", "perjantai", "lauantai"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["tammikuu", "helmikuu", "maaliskuu", "huhtikuu", "toukokuu", "kesäkuu", "heinäkuu", "elokuu", "syyskuu", "lokakuu", "marraskuu", "joulukuu"], ["tammi", "helmi", "maalis", "huhti", "touko", "kesä", "heinä", "elo", "syys", "loka", "marras", "joulu"]],
    standalone_weekdays: [["sunnuntai", "maanantai", "tiistai", "keskiviikko", "torstai", "perjantai", "lauantai"], ["su", "ma", "ti", "ke", "to", "pe", "la"]],
    standalone_periods: ["ap.", "ap.", "ap.", "ap.", "ap.", "ap.", "ap.", "ap.", "ap.", "ap.", "ap.", "ap.", "ip.", "ip.", "ip.", "ip.", "ip.", "ip.", "ip.", "ip.", "ip.", "ip.", "ip.", "ip."],
};

static NUMBERS_72: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "A$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}χιλ.", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}εκ.", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}δισ.", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}τρισ.", forms: &[] }
    ],
};

static DATES_73: Dates = Dates {
    date: ["d/M/yy", "d MMM yyyy", "d MMMM yyyy", "EEEE d MMMM yyyy"],
    time: ["h:mm a", "h:mm:ss a", "h:mm:ss a' UTC'", "h:mm:ss a Συντονισμένη Παγκόσμια Ώρα"],
    datetime: ["d/M/yy, h:mm a", "d MMM yyyy, h:mm a", "d MMMM yyyy στις h:mm a", "EEEE d MMMM yyyy στις h:mm a"],
    months: [["Ιανουαρίου", "Φεβρουαρίου", "Μαρτίου", "Απριλίου", "Μαΐου", "Ιουνίου", "Ιουλίου", "Αυγούστου", "Σεπτεμβρίου", "Οκτωβρίου", "Νοεμβρίου", "Δεκεμβρίου"], ["Ιαν", "Φεβ", "Μαρ", "Απρ", "Μαΐ", "Ιουν", "Ιουλ", "Αυγ", "Σεπ", "Οκτ", "Νοε", "Δεκ"]],
    weekdays: ["Κυριακή", "Δευτέρα", "Τρίτη", "Τετάρτη", "Πέμπτη", "Παρασκευή", "Σάββατο"],
    periods: ["π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ."],
    standalone_months: [["Ιανουαρίου", "Φεβρουαρίου", "Μαρτίου", "Απριλίου", "Μαΐου", "Ιουνίου", "Ιουλίου", "Αυγούστου", "Σεπτεμβρίου", "Οκτωβρίου", "Νοεμβρίου", "Δεκεμβρίου"], ["Ιαν", "Φεβ", "Μαρ", "Απρ", "Μαΐ", "Ιουν", "Ιουλ", "Αυγ", "Σεπ", "Οκτ", "Νοε", "Δεκ"]],
    standalone_weekdays: [["Κυριακή", "Δευτέρα", "Τρίτη", "Τετάρτη", "Πέμπτη", "Παρασκευή", "Σάββατο"], ["Κυρ", "Δευ", "Τρί", "Τετ", "Πέμ", "Παρ", "Σάβ"]],
    standalone_periods: ["π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "π.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ.", "μ.μ."],
};

static NUMBERS_74: Numbers = Numbers {
    zero: '0',
    group: ",",
    decimal: ".",
    minus: "-",
    negative: "\u{200e}-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "\u{200e}-#%",
    currency: "\u{200f}#\u{a0}\u{200f}¤",
    currency_negative: "\u{200f}\u{200e}-#\u{a0}\u{200f}¤",
    currency_group: ",",
    currency_decimal: ".",
    symbols: &[("USD", "$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥\u{200e}"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "A$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#K\u{200f}", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#M\u{200f}", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#B\u{200f}", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#T\u{200f}", forms: &[] }
    ],
};

static DATES_75: Dates = Dates {
    date: ["d.M.yyyy", "d בMMM yyyy", "d בMMMM yyyy", "EEEE, d בMMMM yyyy"],
    time: ["H:mm", "H:mm:ss", "H:mm:ss' UTC'", "H:mm:ss זמן אוניברסלי מתואם"],
    datetime: ["d.M.yyyy, H:mm", "d בMMM yyyy, H:mm", "d בMMMM yyyy בשעה H:mm", "EEEE, d בMMMM yyyy בשעה H:mm"],
    months: [["ינואר", "פברואר", "מרץ", "אפריל", "מאי", "יוני", "יולי", "אוגוסט", "ספטמבר", "אוקטובר", "נובמבר", "דצמבר"], ["ינו׳", "פבר׳", "מרץ", "אפר׳", "מאי", "יוני", "יולי", "אוג׳", "ספט׳", "אוק׳", "נוב׳", "דצמ׳"]],
    weekdays: ["יום ראשון", "יום שני", "יום שלישי", "יום רביעי", "יום חמישי", "יום שישי", "יום שבת"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["ינואר", "פברואר", "מרץ", "אפריל", "מאי", "יוני", "יולי", "אוגוסט", "ספטמבר", "אוקטובר", "נובמבר", "דצמבר"], ["ינו׳", "פבר׳", "מרץ", "אפר׳", "מאי", "יוני", "יולי", "אוג׳", "ספט׳", "אוק׳", "נוב׳", "דצמ׳"]],
    standalone_weekdays: [["יום ראשון", "יום שני", "יום שלישי", "יום רביעי", "יום חמישי", "יום שישי", "יום שבת"], ["יום א׳", "יום ב׳", "יום ג׳", "יום ד׳", "יום ה׳", "יום ו׳", "שבת"]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_76: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "¤#",
    currency_negative: "-¤#",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "JP¥"), ("CNY", "CN¥"), ("INR", "Rs"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}rb", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}jt", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}M", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}T", forms: &[] }
    ],
};

static DATES_77: Dates = Dates {
    date: ["dd/MM/yy", "d MMM yyyy", "d MMMM yyyy", "EEEE, dd MMMM yyyy"],
    time: ["HH.mm", "HH.mm.ss", "HH.mm.ss' UTC'", "HH.mm.ss' Waktu Universal Terkoordinasi'"],
    datetime: ["dd/MM/yy, HH.mm", "d MMM yyyy, HH.mm", "d MMMM yyyy' pukul 'HH.mm", "EEEE, dd MMMM yyyy' pukul 'HH.mm"],
    months: [["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"], ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"]],
    weekdays: ["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"], ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"]],
    standalone_weekdays: [["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"], ["Min", "Sen", "Sel", "Rab", "Kam", "Jum", "Sab"]],
    standalone_periods: ["AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "AM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM", "PM"],
};

static NUMBERS_78: Numbers = Numbers {
    zero: '0',
    group: ".",
    decimal: ",",
    minus: "-",
    negative: "-",
    indian: false,
    min_grouping: 1,
    percent: "#%",
    percent_negative: "-#%",
    currency: "#\u{a0}¤",
    currency_negative: "-#\u{a0}¤",
    currency_group: ".",
    currency_decimal: ",",
    symbols: &[("USD", "US$"), ("EUR", "€"), ("GBP", "£"), ("JPY", "¥"), ("CNY", "CN¥"), ("INR", "₹"), ("KRW", "₩"), ("CAD", "CA$"), ("AUD", "AU$"), ("BRL", "R$"), ("MXN", "MX$"), ("ILS", "₪"), ("VND", "₫")],
    compact: &[
        Compact { magnitude: 3, scale: 3, other: "#\u{a0}N", forms: &[] },
        Compact { magnitude: 6, scale: 6, other: "#\u{a0}Tr", forms: &[] },
        Compact { magnitude: 9, scale: 9, other: "#\u{a0}T", forms: &[] },
        Compact { magnitude: 12, scale: 12, other: "#\u{a0}NT", forms: &[] }
    ],
};

static DATES_79: Dates = Dates {
    date: ["d/M/yy", "d MMM, yyyy", "d MMMM, yyyy", "EEEE, d MMMM, yyyy"],
    time: ["HH:mm", "HH:mm:ss", "HH:mm:ss' UTC'", "HH:mm:ss' Giờ Phối hợp Quốc tế'"],
    datetime: ["HH:mm d/M/yy", "HH:mm d MMM, yyyy", "'lúc 'HH:mm d MMMM, yyyy", "'lúc 'HH:mm EEEE, d MMMM, yyyy"],
    months: [["tháng 1", "tháng 2", "tháng 3", "tháng 4", "tháng 5", "tháng 6", "tháng 7", "tháng 8", "tháng 9", "tháng 10", "tháng 11", "tháng 12"], ["thg 1", "thg 2", "thg 3", "thg 4", "thg 5", "thg 6", "thg 7", "thg 8", "thg 9", "thg 10", "thg 11", "thg 12"]],
    weekdays: ["Chủ Nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy"],
    periods: ["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    standalone_months: [["Tháng 1", "Tháng 2", "Tháng 3", "Tháng 4", "Tháng 5", "Tháng 6", "Tháng 7", "Tháng 8", "Tháng 9", "Tháng 10", "Tháng 11", "Tháng 12"], ["Tháng 1", "Tháng 2", "Tháng 3", "Tháng 4", "Tháng 5", "Tháng 6", "Tháng 7", "Tháng 8", "Tháng 9", "Tháng 10", "Tháng 11", "Tháng 12"]],
    standalone_weekdays: [["Chủ Nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy"], ["CN", "Thứ 2", "Thứ 3", "Thứ 4", "Thứ 5", "Thứ 6", "Thứ 7"]],
    standalone_periods: ["SA", "SA", "SA", "SA", "SA", "SA", "SA", "SA", "SA", "SA", "SA", "SA", "CH", "CH", "CH", "CH", "CH", "CH", "CH", "CH", "CH", "CH", "CH", "CH"],
};

/// Each locale the table knows, by its tag.
static LOCALES: &[(&str, &Numbers, &Dates)] = &[
    ("en", &NUMBERS_0, &DATES_1),
    ("en-GB", &NUMBERS_2, &DATES_3),
    ("en-AU", &NUMBERS_4, &DATES_5),
    ("en-CA", &NUMBERS_6, &DATES_7),
    ("en-IN", &NUMBERS_8, &DATES_9),
    ("de", &NUMBERS_10, &DATES_11),
    ("de-AT", &NUMBERS_12, &DATES_13),
    ("de-CH", &NUMBERS_14, &DATES_11),
    ("fr", &NUMBERS_15, &DATES_16),
    ("fr-CA", &NUMBERS_17, &DATES_18),
    ("fr-CH", &NUMBERS_19, &DATES_20),
    ("es", &NUMBERS_21, &DATES_22),
    ("es-MX", &NUMBERS_23, &DATES_24),
    ("es-419", &NUMBERS_25, &DATES_26),
    ("it", &NUMBERS_27, &DATES_28),
    ("pt", &NUMBERS_29, &DATES_30),
    ("pt-BR", &NUMBERS_29, &DATES_30),
    ("pt-PT", &NUMBERS_31, &DATES_32),
    ("nl", &NUMBERS_33, &DATES_34),
    ("ar", &NUMBERS_35, &DATES_36),
    ("ar-AE", &NUMBERS_35, &DATES_36),
    ("ar-BH", &NUMBERS_37, &DATES_36),
    ("ar-DZ", &NUMBERS_38, &DATES_39),
    ("ar-EG", &NUMBERS_37, &DATES_36),
    ("ar-IQ", &NUMBERS_37, &DATES_40),
    ("ar-JO", &NUMBERS_37, &DATES_40),
    ("ar-KW", &NUMBERS_37, &DATES_36),
    ("ar-LB", &NUMBERS_37, &DATES_40),
    ("ar-LY", &NUMBERS_38, &DATES_36),
    ("ar-MA", &NUMBERS_38, &DATES_41),
    ("ar-OM", &NUMBERS_37, &DATES_36),
    ("ar-PS", &NUMBERS_37, &DATES_40),
    ("ar-QA", &NUMBERS_37, &DATES_36),
    ("ar-SA", &NUMBERS_37, &DATES_36),
    ("ar-SD", &NUMBERS_37, &DATES_36),
    ("ar-SY", &NUMBERS_37, &DATES_40),
    ("ar-TN", &NUMBERS_38, &DATES_39),
    ("ar-YE", &NUMBERS_37, &DATES_36),
    ("ja", &NUMBERS_42, &DATES_43),
    ("zh", &NUMBERS_44, &DATES_45),
    ("zh-TW", &NUMBERS_46, &DATES_47),
    ("zh-HK", &NUMBERS_48, &DATES_49),
    ("zh-Hant", &NUMBERS_46, &DATES_47),
    ("ko", &NUMBERS_50, &DATES_51),
    ("hi", &NUMBERS_52, &DATES_53),
    ("ru", &NUMBERS_54, &DATES_55),
    ("uk", &NUMBERS_56, &DATES_57),
    ("tr", &NUMBERS_58, &DATES_59),
    ("pl", &NUMBERS_60, &DATES_61),
    ("cs", &NUMBERS_62, &DATES_63),
    ("sv", &NUMBERS_64, &DATES_65),
    ("da", &NUMBERS_66, &DATES_67),
    ("nb", &NUMBERS_68, &DATES_69),
    ("fi", &NUMBERS_70, &DATES_71),
    ("el", &NUMBERS_72, &DATES_73),
    ("he", &NUMBERS_74, &DATES_75),
    ("id", &NUMBERS_76, &DATES_77),
    ("vi", &NUMBERS_78, &DATES_79),
];

/// The number symbols an `-u-nu-` extension picks, by language and system.
static NUMBERING: &[(&str, &Numbers)] = &[
    ("ar-u-nu-arab", &NUMBERS_37),
    ("ar-u-nu-latn", &NUMBERS_35),
];

/// The currencies whose amounts are not written with two places.
static CURRENCY_DIGITS: &[(&str, usize)] = &[
    ("JPY", 0),
    ("KRW", 0),
    ("VND", 0),
    ("KWD", 3),
    ("BHD", 3),
    ("OMR", 3),
    ("JOD", 3),
    ("TND", 3),
    ("LYD", 3),
    ("IQD", 0),
    ("CLP", 0),
    ("ISK", 0),
    ("UGX", 0),
    ("XOF", 0),
    ("XAF", 0),
    ("KMF", 0),
    ("PYG", 0),
    ("RWF", 0),
    ("DJF", 0),
    ("GNF", 0),
    ("VUV", 0),
    ("XPF", 0),
    ("BIF", 0),
];
