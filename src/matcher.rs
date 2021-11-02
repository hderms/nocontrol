use memmap::Mmap;
use std::sync::mpsc::SyncSender;

pub struct ControlCharacterMatch {
    pub index: usize,
    pub line: usize,
    pub column: usize,
    pub byte: u8,
}

pub unsafe fn find_data(mmap: &Mmap, tx: SyncSender<ControlCharacterMatch>) {
    let mut line = 0;
    let mut column = 0;
    for (index, byte) in mmap.iter().enumerate() {
        let byte = *byte;

        if !is_printable_character(byte) {
            tx.send(ControlCharacterMatch {
                index,
                line,
                column,
                byte,
            })
            .unwrap();
        }
        if byte == b'\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
}

fn is_printable_character(byte: u8) -> bool {
    !byte.is_ascii_control() || byte == b'\r' || byte == b'\n' || byte == b'\t'
}

#[cfg(test)]
mod test {
    use crate::matcher::is_printable_character;

    #[test]
    fn should_identify_non_printable_characters() {
        assert_eq!(is_printable_character(b'\r'), true);
        assert_eq!(is_printable_character(b'\n'), true);
        assert_eq!(is_printable_character(b'\t'), true);
        assert_eq!(is_printable_character(b' '), true);
        for c in 'a'..='z' {
            assert_eq!(is_printable_character(c as u8), true);
        }

        for c in 'A'..='Z' {
            assert_eq!(is_printable_character(c as u8), true);
        }

        for c in '0'..='9' {
            assert_eq!(is_printable_character(c as u8), true);
        }
    }
}
