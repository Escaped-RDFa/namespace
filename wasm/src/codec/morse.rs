// === Morse ===

pub(crate) const MORSE: &[(char, &str)] = &[
    ('A',".-"),('B',"-..."),('C',"-.-."),('D',"-.."),('E',"."),
    ('F',"..-."),('G',"--."),('H',"...."),('I',".."),('J',".---"),
    ('K',"-.-"),('L',".-.."),('M',"--"),('N',"-."),('O',"---"),
    ('P',".--."),('Q',"--.-"),('R',".-."),('S',"..."),('T',"-"),
    ('U',"..-"),('V',"...-"),('W',".--"),('X',"-..-"),('Y',"-.--"),
    ('Z',"--.."),('0',"-----"),('1',".----"),('2',"..---"),
    ('3',"...--"),('4',"....-"),('5',"....."),('6',"-...."),
    ('7',"--..."),('8',"---.."),('9',"----."),(' ',"/"),
    ('.',".-.-."),(',',"--..-"),('?',"..--.."),('/',"-..-."),
    (':',"---..."),('=',"-..-"),
];

pub(crate) fn char_to_morse(c: char) -> &'static str {
    let u = c.to_ascii_uppercase();
    MORSE.iter().find(|(ch, _)| *ch == u).map(|(_, m)| *m).unwrap_or("?")
}

pub(crate) fn morse_to_char(m: &str) -> char {
    MORSE.iter().find(|(_, code)| *code == m).map(|(c, _)| *c).unwrap_or('?')
}

pub(crate) fn encode_morse(text: &str) -> String {
    text.chars().map(char_to_morse).collect::<Vec<_>>().join(" ")
}

pub(crate) fn decode_morse(morse: &str) -> String {
    morse.split(' ').map(|m| {
        if m == "/" { ' ' } else { morse_to_char(m) }
    }).collect()
}

