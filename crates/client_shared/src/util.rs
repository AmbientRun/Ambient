use rand::seq::SliceRandom;

/// Generates an adjective-animal username.
pub fn random_username() -> String {
    const ADJECTIVES: &[&str] = &[
        "Quirky",
        "Sneaky",
        "Witty",
        "Curious",
        "Grumpy",
        "Silly",
        "Mischievous",
        "Goofy",
        "Hasty",
        "Awkward",
        "Zany",
        "Peculiar",
        "Whimsical",
        "Bumbling",
        "Absurd",
        "Oddball",
        "Clumsy",
        "Nutty",
        "Haphazard",
        "Eccentric",
    ];

    const ANIMALS: &[&str] = &[
        "Penguin",
        "Platypus",
        "Lemur",
        "Armadillo",
        "Sloth",
        "Ostrich",
        "Tapir",
        "Narwhal",
        "Chameleon",
        "Aardvark",
        "Quokka",
        "Wombat",
        "Kakapo",
        "Capybara",
        "Mandrill",
        "Axolotl",
        "Blobfish",
        "Echidna",
        "Wallaby",
    ];

    let mut rng = rand::thread_rng();
    format!(
        "{}{}",
        ADJECTIVES.choose(&mut rng).unwrap(),
        ANIMALS.choose(&mut rng).unwrap()
    )
}
