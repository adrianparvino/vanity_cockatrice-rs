const CARDLIST: &str = r#"
4 Evolving Adaptive
4 Kumano Faces Kakkazan
2 Shivan Devastator
4 Armored Scrapgorger
3 Quirion Beastcaller
2 Arbalest Engineers
4 Bloated Contaminator
3 Kodama of the West Tree
1 Migloz, Maze Crusher
1 Halana and Alena, Partners
3 Thundering Raiju
4 Play with Fire
4 Kami's Flare
1 Lukka, Bound to Ruin
6 Forest
5 Mountain
4 Karplusan Forest
3 Copperline Gorge
1 Boseiju, Who Endures
1 Sokenzan, Crucible of Defiance

2 Strangle
3 Abrade
3 Return to Nature
1 Chandra, Dressed to Kill
4 Rending Flame
2 Jaya, Fiery Negotiator
"#;

use rayon::prelude::*;
use vanity_cockatrice::deck::Deck;

fn main() {
    let f = std::fs::File::open("./names.json").unwrap();
    let cards: Vec<String> = serde_json::from_reader(f).unwrap();
    let cards: Vec<String> = cards.into_iter().map(|card| card.to_lowercase()).collect();
    let cardlist = CARDLIST.to_lowercase();
    let deck = Deck::import(&cardlist).unwrap();
    let mut decks: Vec<Deck> = deck
        .removed()
        .flat_map(|deck| deck.removed())
        .flat_map(|deck| {
            cards.iter().map(move |card| {
                let mut deck = deck.clone();
                deck.insert_sideboard(card);
                deck
            })
        })
        .collect();

    let mut args = std::env::args();
    let prefix = args.nth(1).unwrap();

    let deck = decks
        .par_iter_mut()
        .find_map_any(|deck| {
            let mut buffer = String::new();
            for card in cards.iter() {
                deck.insert_sideboard(card);

                if deck.base32(&mut buffer).starts_with(prefix.as_bytes()) {
                    return Some(deck);
                }

                deck.remove_sideboard(card);
            }

            None
        })
        .unwrap();

    println!(
        "{} {}",
        deck,
        std::str::from_utf8(&deck.base32(&mut String::new())).unwrap()
    );
}
