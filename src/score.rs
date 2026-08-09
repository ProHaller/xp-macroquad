use color_eyre::Result;
use ron::de::SpannedError;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, str::FromStr, time::SystemTime};

pub const HIGHSCORE_PATH: &str = "highscore.ron";

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
pub struct Score {
    pub name: String,
    pub points: u32,
    pub timestamp: SystemTime,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            name: String::from("Anonymous"),
            points: Default::default(),
            timestamp: SystemTime::now(),
        }
    }
}

impl ScoreBoard {
    pub fn best(&self) -> Score {
        self.scores.iter().max().cloned().unwrap_or_default()
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Serialize, Deserialize, Debug, Default)]
pub struct ScoreBoard {
    pub scores: Vec<Score>,
}

impl FromStr for ScoreBoard {
    type Err = SpannedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ron::from_str(s)
    }
}

impl ScoreBoard {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let save_file_content = fs::read_to_string(path)?;
        ron::from_str::<ScoreBoard>(&save_file_content).map_err(|e| e.into())
    }
    pub fn save(&self) -> Result<()> {
        let save_string = ron::to_string(&self)?;
        match fs::write(HIGHSCORE_PATH, save_string) {
            Ok(_) => Ok(()),
            Err(_e) => {
                fs::rename(HIGHSCORE_PATH, HIGHSCORE_PATH.to_owned() + ".bak")?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_serialize() {
        let score = Score {
            name: "test".to_string(),
            points: 999999,
            timestamp: SystemTime::now(),
        };
        let score2 = Score {
            name: "test2".to_string(),
            points: 999999,
            timestamp: SystemTime::now(),
        };
        let scores = vec![score, score2];

        let serialize = ron::to_string(&scores).unwrap();
        fs::write("test.ron", &serialize).unwrap();
        dbg!(&serialize);
        assert_eq!(ron::from_str::<Vec<Score>>(&serialize).unwrap(), scores);
    }

    #[test]
    fn test_load_score() -> Result<()> {
        let scores_str = fs::read_to_string("highscore.ron")?;
        dbg!(&scores_str);
        let parsed: ScoreBoard = ron::from_str(&scores_str)?;
        dbg!(&parsed);
        let scores: Vec<u32> = parsed.scores.iter().map(|s| s.points).collect();
        dbg!(&scores);

        assert!(scores.iter().max().unwrap() > &0u32);
        Ok(())
    }
}
