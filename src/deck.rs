use sha1::{Digest, Sha1};
use std::collections::btree_map::{BTreeMap, Entry};
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Default, Debug)]
pub struct Deck<'s> {
    cached_maindeck: Arc<String>,
    maindeck: Arc<BTreeMap<&'s str, usize>>,
    sideboard: BTreeMap<&'s str, usize>,
}

impl<'s> Deck<'s> {
    pub fn import(buf: &'s str) -> Option<Self> {
        let mut lines = buf.lines().skip_while(|s| s.trim() == "");
        let mut maindeck = BTreeMap::new();
        while let Some(line) = lines.next() {
            if line.trim() == "" {
                break;
            }

            let mut parts = line.splitn(2, ' ');

            let count = parts.next()?;
            let count: usize = count.parse().ok()?;

            let name = parts.next()?;
            maindeck.insert(name, count);
        }

        let cached_maindeck = maindeck
            .iter()
            .flat_map(|(card, n)| std::iter::repeat(*card).take(*n))
            .intersperse(";")
            .collect();

        let mut sideboard = BTreeMap::new();
        while let Some(line) = lines.next() {
            if line.trim() == "" {
                break;
            }

            let mut parts = line.splitn(2, ' ');

            let count = parts.next()?;
            let count: usize = count.parse().ok()?;

            let name = parts.next()?;
            sideboard.insert(name, count);
        }

        Some(Deck {
            maindeck: Arc::new(maindeck),
            cached_maindeck: Arc::new(cached_maindeck),
            sideboard,
        })
    }

    #[inline(always)]
    pub fn base32(&self, buffer: &mut String) -> [u8; 8] {
        let mut hasher = Sha1::new();
        for (card, n) in &self.sideboard {
            for _ in 0..*n {
                hasher.update("SB:");
                hasher.update(card);
                hasher.update(";");
            }
        }
        hasher.update(buffer);

        hasher.update(&*self.cached_maindeck);

        let hash = hasher.finalize();

        unsafe {
            use std::arch::asm;

            let mut word = u64::from_be_bytes(hash[0..8].try_into().unwrap());
            word >>= 24;

            asm!(
                "pdep {word}, {word}, {mask}",
                word = inout(reg) word,
                mask = in(reg) 0x1F1F1F1F1F1F1F1Fu64
            );
            let mask: u64 = (word + (0x8080808080808080 - 0x0a0a0a0a0a0a0a0a)) & 0x8080808080808080;

            let select = ((((mask as u128) * ((((b'a' - 10 - b'0') as u64) << 57) as u128)) >> 64)
                as u64)
                + 0x3030303030303030;

            let word = word + select;

            word.to_be_bytes()
        }
    }

    pub fn removed(self) -> Vec<Deck<'s>> {
        let keys = self.sideboard.clone().into_keys();
        let mut decks = Vec::new();

        for key in keys {
            let mut deck = self.clone();
            let entry = deck.sideboard.entry(key);
            match entry {
                Entry::Occupied(mut entry) => {
                    let x = entry.get_mut();
                    *x -= 1;
                    if *x == 0 {
                        entry.remove_entry();
                    }
                }
                _ => unreachable!(),
            };

            decks.push(deck);
        }

        decks
    }

    pub fn insert_sideboard(&mut self, card: &'s str) {
        *self.sideboard.entry(card).or_insert(0) += 1;
    }

    pub fn remove_sideboard(&mut self, card: &'s str) {
        let entry = self.sideboard.entry(card);
        match entry {
            Entry::Occupied(mut entry) => {
                let x = entry.get_mut();
                *x -= 1;
                if *x == 0 {
                    entry.remove_entry();
                }
            }
            _ => unreachable!(),
        };
    }
}

impl<'s> fmt::Display for Deck<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (card, n) in &*self.maindeck {
            write!(f, "{} {}\n", n, card)?;
        }

        write!(f, "\n")?;

        for (card, n) in &self.sideboard {
            write!(f, "{} {}\n", n, card)?;
        }

        Ok(())
    }
}
