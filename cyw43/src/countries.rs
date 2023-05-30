#![allow(unused)]

pub struct Country {
    pub code: [u8; 2],
    pub rev: u16,
}

/// AF Afghanistan
pub const AFGHANISTAN: Country = Country { code: *b"AF", rev: 0 };
/// AL Albania
pub const ALBANIA: Country = Country { code: *b"AL", rev: 0 };
/// DZ Algeria
pub const ALGERIA: Country = Country { code: *b"DZ", rev: 0 };
/// AS American_Samoa
pub const AMERICAN_SAMOA: Country = Country { code: *b"AS", rev: 0 };
/// AO Angola
pub const ANGOLA: Country = Country { code: *b"AO", rev: 0 };
/// AI Anguilla
pub const ANGUILLA: Country = Country { code: *b"AI", rev: 0 };
/// AG Antigua_and_Barbuda
pub const ANTIGUA_AND_BARBUDA: Country = Country { code: *b"AG", rev: 0 };
/// AR Argentina
pub const ARGENTINA: Country = Country { code: *b"AR", rev: 0 };
/// AM Armenia
pub const ARMENIA: Country = Country { code: *b"AM", rev: 0 };
/// AW Aruba
pub const ARUBA: Country = Country { code: *b"AW", rev: 0 };
/// AU Australia
pub const AUSTRALIA: Country = Country { code: *b"AU", rev: 0 };
/// AT Austria
pub const AUSTRIA: Country = Country { code: *b"AT", rev: 0 };
/// AZ Azerbaijan
pub const AZERBAIJAN: Country = Country { code: *b"AZ", rev: 0 };
/// BS Bahamas
pub const BAHAMAS: Country = Country { code: *b"BS", rev: 0 };
/// BH Bahrain
pub const BAHRAIN: Country = Country { code: *b"BH", rev: 0 };
/// 0B Baker_Island
pub const BAKER_ISLAND: Country = Country { code: *b"0B", rev: 0 };
/// BD Bangladesh
pub const BANGLADESH: Country = Country { code: *b"BD", rev: 0 };
/// BB Barbados
pub const BARBADOS: Country = Country { code: *b"BB", rev: 0 };
/// BY Belarus
pub const BELARUS: Country = Country { code: *b"BY", rev: 0 };
/// BE Belgium
pub const BELGIUM: Country = Country { code: *b"BE", rev: 0 };
/// BZ Belize
pub const BELIZE: Country = Country { code: *b"BZ", rev: 0 };
/// BJ Benin
pub const BENIN: Country = Country { code: *b"BJ", rev: 0 };
/// BM Bermuda
pub const BERMUDA: Country = Country { code: *b"BM", rev: 0 };
/// BT Bhutan
pub const BHUTAN: Country = Country { code: *b"BT", rev: 0 };
/// BO Bolivia
pub const BOLIVIA: Country = Country { code: *b"BO", rev: 0 };
/// BA Bosnia_and_Herzegovina
pub const BOSNIA_AND_HERZEGOVINA: Country = Country { code: *b"BA", rev: 0 };
/// BW Botswana
pub const BOTSWANA: Country = Country { code: *b"BW", rev: 0 };
/// BR Brazil
pub const BRAZIL: Country = Country { code: *b"BR", rev: 0 };
/// IO British_Indian_Ocean_Territory
pub const BRITISH_INDIAN_OCEAN_TERRITORY: Country = Country { code: *b"IO", rev: 0 };
/// BN Brunei_Darussalam
pub const BRUNEI_DARUSSALAM: Country = Country { code: *b"BN", rev: 0 };
/// BG Bulgaria
pub const BULGARIA: Country = Country { code: *b"BG", rev: 0 };
/// BF Burkina_Faso
pub const BURKINA_FASO: Country = Country { code: *b"BF", rev: 0 };
/// BI Burundi
pub const BURUNDI: Country = Country { code: *b"BI", rev: 0 };
/// KH Cambodia
pub const CAMBODIA: Country = Country { code: *b"KH", rev: 0 };
/// CM Cameroon
pub const CAMEROON: Country = Country { code: *b"CM", rev: 0 };
/// CA Canada
pub const CANADA: Country = Country { code: *b"CA", rev: 0 };
/// CA Canada Revision 950
pub const CANADA_REV950: Country = Country { code: *b"CA", rev: 950 };
/// CV Cape_Verde
pub const CAPE_VERDE: Country = Country { code: *b"CV", rev: 0 };
/// KY Cayman_Islands
pub const CAYMAN_ISLANDS: Country = Country { code: *b"KY", rev: 0 };
/// CF Central_African_Republic
pub const CENTRAL_AFRICAN_REPUBLIC: Country = Country { code: *b"CF", rev: 0 };
/// TD Chad
pub const CHAD: Country = Country { code: *b"TD", rev: 0 };
/// CL Chile
pub const CHILE: Country = Country { code: *b"CL", rev: 0 };
/// CN China
pub const CHINA: Country = Country { code: *b"CN", rev: 0 };
/// CX Christmas_Island
pub const CHRISTMAS_ISLAND: Country = Country { code: *b"CX", rev: 0 };
/// CO Colombia
pub const COLOMBIA: Country = Country { code: *b"CO", rev: 0 };
/// KM Comoros
pub const COMOROS: Country = Country { code: *b"KM", rev: 0 };
/// CG Congo
pub const CONGO: Country = Country { code: *b"CG", rev: 0 };
/// CD Congo,_The_Democratic_Republic_Of_The
pub const CONGO_THE_DEMOCRATIC_REPUBLIC_OF_THE: Country = Country { code: *b"CD", rev: 0 };
/// CR Costa_Rica
pub const COSTA_RICA: Country = Country { code: *b"CR", rev: 0 };
/// CI Cote_D'ivoire
pub const COTE_DIVOIRE: Country = Country { code: *b"CI", rev: 0 };
/// HR Croatia
pub const CROATIA: Country = Country { code: *b"HR", rev: 0 };
/// CU Cuba
pub const CUBA: Country = Country { code: *b"CU", rev: 0 };
/// CY Cyprus
pub const CYPRUS: Country = Country { code: *b"CY", rev: 0 };
/// CZ Czech_Republic
pub const CZECH_REPUBLIC: Country = Country { code: *b"CZ", rev: 0 };
/// DK Denmark
pub const DENMARK: Country = Country { code: *b"DK", rev: 0 };
/// DJ Djibouti
pub const DJIBOUTI: Country = Country { code: *b"DJ", rev: 0 };
/// DM Dominica
pub const DOMINICA: Country = Country { code: *b"DM", rev: 0 };
/// DO Dominican_Republic
pub const DOMINICAN_REPUBLIC: Country = Country { code: *b"DO", rev: 0 };
/// AU G'Day mate!
pub const DOWN_UNDER: Country = Country { code: *b"AU", rev: 0 };
/// EC Ecuador
pub const ECUADOR: Country = Country { code: *b"EC", rev: 0 };
/// EG Egypt
pub const EGYPT: Country = Country { code: *b"EG", rev: 0 };
/// SV El_Salvador
pub const EL_SALVADOR: Country = Country { code: *b"SV", rev: 0 };
/// GQ Equatorial_Guinea
pub const EQUATORIAL_GUINEA: Country = Country { code: *b"GQ", rev: 0 };
/// ER Eritrea
pub const ERITREA: Country = Country { code: *b"ER", rev: 0 };
/// EE Estonia
pub const ESTONIA: Country = Country { code: *b"EE", rev: 0 };
/// ET Ethiopia
pub const ETHIOPIA: Country = Country { code: *b"ET", rev: 0 };
/// FK Falkland_Islands_(Malvinas)
pub const FALKLAND_ISLANDS_MALVINAS: Country = Country { code: *b"FK", rev: 0 };
/// FO Faroe_Islands
pub const FAROE_ISLANDS: Country = Country { code: *b"FO", rev: 0 };
/// FJ Fiji
pub const FIJI: Country = Country { code: *b"FJ", rev: 0 };
/// FI Finland
pub const FINLAND: Country = Country { code: *b"FI", rev: 0 };
/// FR France
pub const FRANCE: Country = Country { code: *b"FR", rev: 0 };
/// GF French_Guina
pub const FRENCH_GUINA: Country = Country { code: *b"GF", rev: 0 };
/// PF French_Polynesia
pub const FRENCH_POLYNESIA: Country = Country { code: *b"PF", rev: 0 };
/// TF French_Southern_Territories
pub const FRENCH_SOUTHERN_TERRITORIES: Country = Country { code: *b"TF", rev: 0 };
/// GA Gabon
pub const GABON: Country = Country { code: *b"GA", rev: 0 };
/// GM Gambia
pub const GAMBIA: Country = Country { code: *b"GM", rev: 0 };
/// GE Georgia
pub const GEORGIA: Country = Country { code: *b"GE", rev: 0 };
/// DE Germany
pub const GERMANY: Country = Country { code: *b"DE", rev: 0 };
/// E0 European_Wide Revision 895
pub const EUROPEAN_WIDE_REV895: Country = Country { code: *b"E0", rev: 895 };
/// GH Ghana
pub const GHANA: Country = Country { code: *b"GH", rev: 0 };
/// GI Gibraltar
pub const GIBRALTAR: Country = Country { code: *b"GI", rev: 0 };
/// GR Greece
pub const GREECE: Country = Country { code: *b"GR", rev: 0 };
/// GD Grenada
pub const GRENADA: Country = Country { code: *b"GD", rev: 0 };
/// GP Guadeloupe
pub const GUADELOUPE: Country = Country { code: *b"GP", rev: 0 };
/// GU Guam
pub const GUAM: Country = Country { code: *b"GU", rev: 0 };
/// GT Guatemala
pub const GUATEMALA: Country = Country { code: *b"GT", rev: 0 };
/// GG Guernsey
pub const GUERNSEY: Country = Country { code: *b"GG", rev: 0 };
/// GN Guinea
pub const GUINEA: Country = Country { code: *b"GN", rev: 0 };
/// GW Guinea-bissau
pub const GUINEA_BISSAU: Country = Country { code: *b"GW", rev: 0 };
/// GY Guyana
pub const GUYANA: Country = Country { code: *b"GY", rev: 0 };
/// HT Haiti
pub const HAITI: Country = Country { code: *b"HT", rev: 0 };
/// VA Holy_See_(Vatican_City_State)
pub const HOLY_SEE_VATICAN_CITY_STATE: Country = Country { code: *b"VA", rev: 0 };
/// HN Honduras
pub const HONDURAS: Country = Country { code: *b"HN", rev: 0 };
/// HK Hong_Kong
pub const HONG_KONG: Country = Country { code: *b"HK", rev: 0 };
/// HU Hungary
pub const HUNGARY: Country = Country { code: *b"HU", rev: 0 };
/// IS Iceland
pub const ICELAND: Country = Country { code: *b"IS", rev: 0 };
/// IN India
pub const INDIA: Country = Country { code: *b"IN", rev: 0 };
/// ID Indonesia
pub const INDONESIA: Country = Country { code: *b"ID", rev: 0 };
/// IR Iran,_Islamic_Republic_Of
pub const IRAN_ISLAMIC_REPUBLIC_OF: Country = Country { code: *b"IR", rev: 0 };
/// IQ Iraq
pub const IRAQ: Country = Country { code: *b"IQ", rev: 0 };
/// IE Ireland
pub const IRELAND: Country = Country { code: *b"IE", rev: 0 };
/// IL Israel
pub const ISRAEL: Country = Country { code: *b"IL", rev: 0 };
/// IT Italy
pub const ITALY: Country = Country { code: *b"IT", rev: 0 };
/// JM Jamaica
pub const JAMAICA: Country = Country { code: *b"JM", rev: 0 };
/// JP Japan
pub const JAPAN: Country = Country { code: *b"JP", rev: 0 };
/// JE Jersey
pub const JERSEY: Country = Country { code: *b"JE", rev: 0 };
/// JO Jordan
pub const JORDAN: Country = Country { code: *b"JO", rev: 0 };
/// KZ Kazakhstan
pub const KAZAKHSTAN: Country = Country { code: *b"KZ", rev: 0 };
/// KE Kenya
pub const KENYA: Country = Country { code: *b"KE", rev: 0 };
/// KI Kiribati
pub const KIRIBATI: Country = Country { code: *b"KI", rev: 0 };
/// KR Korea,_Republic_Of
pub const KOREA_REPUBLIC_OF: Country = Country { code: *b"KR", rev: 1 };
/// 0A Kosovo
pub const KOSOVO: Country = Country { code: *b"0A", rev: 0 };
/// KW Kuwait
pub const KUWAIT: Country = Country { code: *b"KW", rev: 0 };
/// KG Kyrgyzstan
pub const KYRGYZSTAN: Country = Country { code: *b"KG", rev: 0 };
/// LA Lao_People's_Democratic_Repubic
pub const LAO_PEOPLES_DEMOCRATIC_REPUBIC: Country = Country { code: *b"LA", rev: 0 };
/// LV Latvia
pub const LATVIA: Country = Country { code: *b"LV", rev: 0 };
/// LB Lebanon
pub const LEBANON: Country = Country { code: *b"LB", rev: 0 };
/// LS Lesotho
pub const LESOTHO: Country = Country { code: *b"LS", rev: 0 };
/// LR Liberia
pub const LIBERIA: Country = Country { code: *b"LR", rev: 0 };
/// LY Libyan_Arab_Jamahiriya
pub const LIBYAN_ARAB_JAMAHIRIYA: Country = Country { code: *b"LY", rev: 0 };
/// LI Liechtenstein
pub const LIECHTENSTEIN: Country = Country { code: *b"LI", rev: 0 };
/// LT Lithuania
pub const LITHUANIA: Country = Country { code: *b"LT", rev: 0 };
/// LU Luxembourg
pub const LUXEMBOURG: Country = Country { code: *b"LU", rev: 0 };
/// MO Macao
pub const MACAO: Country = Country { code: *b"MO", rev: 0 };
/// MK Macedonia,_Former_Yugoslav_Republic_Of
pub const MACEDONIA_FORMER_YUGOSLAV_REPUBLIC_OF: Country = Country { code: *b"MK", rev: 0 };
/// MG Madagascar
pub const MADAGASCAR: Country = Country { code: *b"MG", rev: 0 };
/// MW Malawi
pub const MALAWI: Country = Country { code: *b"MW", rev: 0 };
/// MY Malaysia
pub const MALAYSIA: Country = Country { code: *b"MY", rev: 0 };
/// MV Maldives
pub const MALDIVES: Country = Country { code: *b"MV", rev: 0 };
/// ML Mali
pub const MALI: Country = Country { code: *b"ML", rev: 0 };
/// MT Malta
pub const MALTA: Country = Country { code: *b"MT", rev: 0 };
/// IM Man,_Isle_Of
pub const MAN_ISLE_OF: Country = Country { code: *b"IM", rev: 0 };
/// MQ Martinique
pub const MARTINIQUE: Country = Country { code: *b"MQ", rev: 0 };
/// MR Mauritania
pub const MAURITANIA: Country = Country { code: *b"MR", rev: 0 };
/// MU Mauritius
pub const MAURITIUS: Country = Country { code: *b"MU", rev: 0 };
/// YT Mayotte
pub const MAYOTTE: Country = Country { code: *b"YT", rev: 0 };
/// MX Mexico
pub const MEXICO: Country = Country { code: *b"MX", rev: 0 };
/// FM Micronesia,_Federated_States_Of
pub const MICRONESIA_FEDERATED_STATES_OF: Country = Country { code: *b"FM", rev: 0 };
/// MD Moldova,_Republic_Of
pub const MOLDOVA_REPUBLIC_OF: Country = Country { code: *b"MD", rev: 0 };
/// MC Monaco
pub const MONACO: Country = Country { code: *b"MC", rev: 0 };
/// MN Mongolia
pub const MONGOLIA: Country = Country { code: *b"MN", rev: 0 };
/// ME Montenegro
pub const MONTENEGRO: Country = Country { code: *b"ME", rev: 0 };
/// MS Montserrat
pub const MONTSERRAT: Country = Country { code: *b"MS", rev: 0 };
/// MA Morocco
pub const MOROCCO: Country = Country { code: *b"MA", rev: 0 };
/// MZ Mozambique
pub const MOZAMBIQUE: Country = Country { code: *b"MZ", rev: 0 };
/// MM Myanmar
pub const MYANMAR: Country = Country { code: *b"MM", rev: 0 };
/// NA Namibia
pub const NAMIBIA: Country = Country { code: *b"NA", rev: 0 };
/// NR Nauru
pub const NAURU: Country = Country { code: *b"NR", rev: 0 };
/// NP Nepal
pub const NEPAL: Country = Country { code: *b"NP", rev: 0 };
/// NL Netherlands
pub const NETHERLANDS: Country = Country { code: *b"NL", rev: 0 };
/// AN Netherlands_Antilles
pub const NETHERLANDS_ANTILLES: Country = Country { code: *b"AN", rev: 0 };
/// NC New_Caledonia
pub const NEW_CALEDONIA: Country = Country { code: *b"NC", rev: 0 };
/// NZ New_Zealand
pub const NEW_ZEALAND: Country = Country { code: *b"NZ", rev: 0 };
/// NI Nicaragua
pub const NICARAGUA: Country = Country { code: *b"NI", rev: 0 };
/// NE Niger
pub const NIGER: Country = Country { code: *b"NE", rev: 0 };
/// NG Nigeria
pub const NIGERIA: Country = Country { code: *b"NG", rev: 0 };
/// NF Norfolk_Island
pub const NORFOLK_ISLAND: Country = Country { code: *b"NF", rev: 0 };
/// MP Northern_Mariana_Islands
pub const NORTHERN_MARIANA_ISLANDS: Country = Country { code: *b"MP", rev: 0 };
/// NO Norway
pub const NORWAY: Country = Country { code: *b"NO", rev: 0 };
/// OM Oman
pub const OMAN: Country = Country { code: *b"OM", rev: 0 };
/// PK Pakistan
pub const PAKISTAN: Country = Country { code: *b"PK", rev: 0 };
/// PW Palau
pub const PALAU: Country = Country { code: *b"PW", rev: 0 };
/// PA Panama
pub const PANAMA: Country = Country { code: *b"PA", rev: 0 };
/// PG Papua_New_Guinea
pub const PAPUA_NEW_GUINEA: Country = Country { code: *b"PG", rev: 0 };
/// PY Paraguay
pub const PARAGUAY: Country = Country { code: *b"PY", rev: 0 };
/// PE Peru
pub const PERU: Country = Country { code: *b"PE", rev: 0 };
/// PH Philippines
pub const PHILIPPINES: Country = Country { code: *b"PH", rev: 0 };
/// PL Poland
pub const POLAND: Country = Country { code: *b"PL", rev: 0 };
/// PT Portugal
pub const PORTUGAL: Country = Country { code: *b"PT", rev: 0 };
/// PR Pueto_Rico
pub const PUETO_RICO: Country = Country { code: *b"PR", rev: 0 };
/// QA Qatar
pub const QATAR: Country = Country { code: *b"QA", rev: 0 };
/// RE Reunion
pub const REUNION: Country = Country { code: *b"RE", rev: 0 };
/// RO Romania
pub const ROMANIA: Country = Country { code: *b"RO", rev: 0 };
/// RU Russian_Federation
pub const RUSSIAN_FEDERATION: Country = Country { code: *b"RU", rev: 0 };
/// RW Rwanda
pub const RWANDA: Country = Country { code: *b"RW", rev: 0 };
/// KN Saint_Kitts_and_Nevis
pub const SAINT_KITTS_AND_NEVIS: Country = Country { code: *b"KN", rev: 0 };
/// LC Saint_Lucia
pub const SAINT_LUCIA: Country = Country { code: *b"LC", rev: 0 };
/// PM Saint_Pierre_and_Miquelon
pub const SAINT_PIERRE_AND_MIQUELON: Country = Country { code: *b"PM", rev: 0 };
/// VC Saint_Vincent_and_The_Grenadines
pub const SAINT_VINCENT_AND_THE_GRENADINES: Country = Country { code: *b"VC", rev: 0 };
/// WS Samoa
pub const SAMOA: Country = Country { code: *b"WS", rev: 0 };
/// MF Sanit_Martin_/_Sint_Marteen
pub const SANIT_MARTIN_SINT_MARTEEN: Country = Country { code: *b"MF", rev: 0 };
/// ST Sao_Tome_and_Principe
pub const SAO_TOME_AND_PRINCIPE: Country = Country { code: *b"ST", rev: 0 };
/// SA Saudi_Arabia
pub const SAUDI_ARABIA: Country = Country { code: *b"SA", rev: 0 };
/// SN Senegal
pub const SENEGAL: Country = Country { code: *b"SN", rev: 0 };
/// RS Serbia
pub const SERBIA: Country = Country { code: *b"RS", rev: 0 };
/// SC Seychelles
pub const SEYCHELLES: Country = Country { code: *b"SC", rev: 0 };
/// SL Sierra_Leone
pub const SIERRA_LEONE: Country = Country { code: *b"SL", rev: 0 };
/// SG Singapore
pub const SINGAPORE: Country = Country { code: *b"SG", rev: 0 };
/// SK Slovakia
pub const SLOVAKIA: Country = Country { code: *b"SK", rev: 0 };
/// SI Slovenia
pub const SLOVENIA: Country = Country { code: *b"SI", rev: 0 };
/// SB Solomon_Islands
pub const SOLOMON_ISLANDS: Country = Country { code: *b"SB", rev: 0 };
/// SO Somalia
pub const SOMALIA: Country = Country { code: *b"SO", rev: 0 };
/// ZA South_Africa
pub const SOUTH_AFRICA: Country = Country { code: *b"ZA", rev: 0 };
/// ES Spain
pub const SPAIN: Country = Country { code: *b"ES", rev: 0 };
/// LK Sri_Lanka
pub const SRI_LANKA: Country = Country { code: *b"LK", rev: 0 };
/// SR Suriname
pub const SURINAME: Country = Country { code: *b"SR", rev: 0 };
/// SZ Swaziland
pub const SWAZILAND: Country = Country { code: *b"SZ", rev: 0 };
/// SE Sweden
pub const SWEDEN: Country = Country { code: *b"SE", rev: 0 };
/// CH Switzerland
pub const SWITZERLAND: Country = Country { code: *b"CH", rev: 0 };
/// SY Syrian_Arab_Republic
pub const SYRIAN_ARAB_REPUBLIC: Country = Country { code: *b"SY", rev: 0 };
/// TW Taiwan,_Province_Of_China
pub const TAIWAN_PROVINCE_OF_CHINA: Country = Country { code: *b"TW", rev: 0 };
/// TJ Tajikistan
pub const TAJIKISTAN: Country = Country { code: *b"TJ", rev: 0 };
/// TZ Tanzania,_United_Republic_Of
pub const TANZANIA_UNITED_REPUBLIC_OF: Country = Country { code: *b"TZ", rev: 0 };
/// TH Thailand
pub const THAILAND: Country = Country { code: *b"TH", rev: 0 };
/// TG Togo
pub const TOGO: Country = Country { code: *b"TG", rev: 0 };
/// TO Tonga
pub const TONGA: Country = Country { code: *b"TO", rev: 0 };
/// TT Trinidad_and_Tobago
pub const TRINIDAD_AND_TOBAGO: Country = Country { code: *b"TT", rev: 0 };
/// TN Tunisia
pub const TUNISIA: Country = Country { code: *b"TN", rev: 0 };
/// TR Turkey
pub const TURKEY: Country = Country { code: *b"TR", rev: 0 };
/// TM Turkmenistan
pub const TURKMENISTAN: Country = Country { code: *b"TM", rev: 0 };
/// TC Turks_and_Caicos_Islands
pub const TURKS_AND_CAICOS_ISLANDS: Country = Country { code: *b"TC", rev: 0 };
/// TV Tuvalu
pub const TUVALU: Country = Country { code: *b"TV", rev: 0 };
/// UG Uganda
pub const UGANDA: Country = Country { code: *b"UG", rev: 0 };
/// UA Ukraine
pub const UKRAINE: Country = Country { code: *b"UA", rev: 0 };
/// AE United_Arab_Emirates
pub const UNITED_ARAB_EMIRATES: Country = Country { code: *b"AE", rev: 0 };
/// GB United_Kingdom
pub const UNITED_KINGDOM: Country = Country { code: *b"GB", rev: 0 };
/// US United_States
pub const UNITED_STATES: Country = Country { code: *b"US", rev: 0 };
/// US United_States Revision 4
pub const UNITED_STATES_REV4: Country = Country { code: *b"US", rev: 4 };
/// Q1 United_States Revision 931
pub const UNITED_STATES_REV931: Country = Country { code: *b"Q1", rev: 931 };
/// Q2 United_States_(No_DFS)
pub const UNITED_STATES_NO_DFS: Country = Country { code: *b"Q2", rev: 0 };
/// UM United_States_Minor_Outlying_Islands
pub const UNITED_STATES_MINOR_OUTLYING_ISLANDS: Country = Country { code: *b"UM", rev: 0 };
/// UY Uruguay
pub const URUGUAY: Country = Country { code: *b"UY", rev: 0 };
/// UZ Uzbekistan
pub const UZBEKISTAN: Country = Country { code: *b"UZ", rev: 0 };
/// VU Vanuatu
pub const VANUATU: Country = Country { code: *b"VU", rev: 0 };
/// VE Venezuela
pub const VENEZUELA: Country = Country { code: *b"VE", rev: 0 };
/// VN Viet_Nam
pub const VIET_NAM: Country = Country { code: *b"VN", rev: 0 };
/// VG Virgin_Islands,_British
pub const VIRGIN_ISLANDS_BRITISH: Country = Country { code: *b"VG", rev: 0 };
/// VI Virgin_Islands,_U.S.
pub const VIRGIN_ISLANDS_US: Country = Country { code: *b"VI", rev: 0 };
/// WF Wallis_and_Futuna
pub const WALLIS_AND_FUTUNA: Country = Country { code: *b"WF", rev: 0 };
/// 0C West_Bank
pub const WEST_BANK: Country = Country { code: *b"0C", rev: 0 };
/// EH Western_Sahara
pub const WESTERN_SAHARA: Country = Country { code: *b"EH", rev: 0 };
/// Worldwide Locale Revision 983
pub const WORLD_WIDE_XV_REV983: Country = Country { code: *b"XV", rev: 983 };
/// Worldwide Locale (passive Ch12-14)
pub const WORLD_WIDE_XX: Country = Country { code: *b"XX", rev: 0 };
/// Worldwide Locale (passive Ch12-14) Revision 17
pub const WORLD_WIDE_XX_REV17: Country = Country { code: *b"XX", rev: 17 };
/// YE Yemen
pub const YEMEN: Country = Country { code: *b"YE", rev: 0 };
/// ZM Zambia
pub const ZAMBIA: Country = Country { code: *b"ZM", rev: 0 };
/// ZW Zimbabwe
pub const ZIMBABWE: Country = Country { code: *b"ZW", rev: 0 };
