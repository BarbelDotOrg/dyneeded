const BIBLE_PASSAGES: &'static [&'static str] = &[
    "O Jerusalem, Jerusalem, the city that kills the prophets and stones those who are sent to it! How often would I have gathered your children together as a hen gathers her brood under her wings, and you were not willing!",
    "One who loves money will not be satisfied with money, nor one who loves abundance with its income. This too is futility.",
    "And they overcame him because of the blood of the Lamb and because of the word of their testimony, and they did not love their life even when faced with death.",
    "And the Light shines in the darkness, and the darkness did not comprehend it.",
    "If you were of the world, the world would love you as its own; but because you are not of the world, but I chose you out of the world, because of this the world hates you.",
    "Man shall not live on bread alone, but on every word that comes out of the mouth of God.",
    "The spirit is willing, but the flesh is weak.",
    "Blessed are those who hunger and thirst for righteousness, for they will be satisfied.",
    "Blessed are the merciful, for they will receive mercy.",
    "Blessed are those who have been persecuted for the sake of righteousness, for theirs is the kingdom of heaven.",
    "Even though I walk through the valley of the shadow of death, I fear no evil, for You are with me; Your rod and Your staff, they comfort me.",
    "He was despised and abandoned by men, A man of great pain and familiar with sickness; And like one from whom people hide their faces, He was despised, and we had no regard for Him.",
    "What good is it for someone to gain the whole world, yet forfeit their soul?",
    "For whoever wants to save his life will lose it, but whoever loses his life for My sake and for the gospel will save it.",
    "If you continue in My word, then you are truly My disciples; and you will know the truth, and the truth will set you free."
];

pub fn random_bible_passage() -> &'static str {
    BIBLE_PASSAGES[fastrand::usize(0..BIBLE_PASSAGES.len() - 1)]
}
