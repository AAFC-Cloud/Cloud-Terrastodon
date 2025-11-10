use compact_str::CompactString;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LocationName {
    Asia,
    AsiaPacific,
    Australia,
    AustraliaCentral,
    AustraliaCentral2,
    AustraliaEast,
    AustraliaSoutheast,
    AustriaEast,
    BelgiumCentral,
    Brazil,
    BrazilSouth,
    BrazilSoutheast,
    BrazilUS,
    Canada,
    CanadaCentral,
    CanadaEast,
    CentralIndia,
    CentralUS,
    CentralUSEUAP,
    CentralUSStage,
    ChileCentral,
    EastAsia,
    EastAsiaStage,
    EastUS,
    EastUS2,
    EastUS2EUAP,
    EastUS2Stage,
    EastUSStage,
    EastUSSTG,
    Europe,
    France,
    FranceCentral,
    FranceSouth,
    Germany,
    GermanyNorth,
    GermanyWestCentral,
    Global,
    India,
    Indonesia,
    IndonesiaCentral,
    Israel,
    IsraelCentral,
    Italy,
    ItalyNorth,
    Japan,
    JapanEast,
    JapanWest,
    JioIndiaCentral,
    JioIndiaWest,
    Korea,
    KoreaCentral,
    KoreaSouth,
    Malaysia,
    MalaysiaWest,
    Mexico,
    MexicoCentral,
    NewZealand,
    NewZealandNorth,
    NorthCentralUS,
    NorthCentralUSStage,
    NorthEurope,
    Norway,
    NorwayEast,
    NorwayWest,
    Poland,
    PolandCentral,
    Qatar,
    QatarCentral,
    Singapore,
    SouthAfrica,
    SouthAfricaNorth,
    SouthAfricaWest,
    SouthCentralUS,
    SouthCentralUSStage,
    SouthCentralUSSTG,
    SoutheastAsia,
    SoutheastAsiaStage,
    SouthIndia,
    Spain,
    SpainCentral,
    Sweden,
    SwedenCentral,
    SwedenSouth,
    Switzerland,
    SwitzerlandNorth,
    SwitzerlandWest,
    Taiwan,
    UAE,
    UAECentral,
    UAENorth,
    UK,
    UKSouth,
    UKWest,
    UnitedArabEmirates,
    UnitedKingdom,
    UnitedStates,
    UnitedStatesEUAP,
    WestCentralUS,
    WestEurope,
    WestIndia,
    WestUS,
    WestUS2,
    WestUS2Stage,
    WestUS3,
    WestUSStage,
    Other(String),
}
impl LocationName {
    pub fn is_canada(&self) -> bool {
        matches!(
            self,
            LocationName::Canada | LocationName::CanadaCentral | LocationName::CanadaEast
        )
    }
    pub fn as_other(&self) -> Option<&str> {
        if let LocationName::Other(name) = self {
            Some(name)
        } else {
            None
        }
    }
}
impl std::fmt::Display for LocationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocationName::Other(name) => f.write_str(name),
            _ => f.write_str(&format!("{self:?}").to_ascii_lowercase()),
        }
    }
}
impl FromStr for LocationName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "asia" => LocationName::Asia,
            "asiapacific" => LocationName::AsiaPacific,
            "australia" => LocationName::Australia,
            "australiacentral" => LocationName::AustraliaCentral,
            "australiacentral2" => LocationName::AustraliaCentral2,
            "australiaeast" => LocationName::AustraliaEast,
            "australiasoutheast" => LocationName::AustraliaSoutheast,
            "austriaeast" => LocationName::AustriaEast,
            "belgiumcentral" => LocationName::BelgiumCentral,
            "brazil" => LocationName::Brazil,
            "brazilsouth" => LocationName::BrazilSouth,
            "brazilsoutheast" => LocationName::BrazilSoutheast,
            "brazilus" => LocationName::BrazilUS,
            "canada" => LocationName::Canada,
            "canadacentral" => LocationName::CanadaCentral,
            "canadaeast" => LocationName::CanadaEast,
            "centralindia" => LocationName::CentralIndia,
            "centralus" => LocationName::CentralUS,
            "centraluseuap" => LocationName::CentralUSEUAP,
            "centralusstage" => LocationName::CentralUSStage,
            "chilecentral" => LocationName::ChileCentral,
            "eastasia" => LocationName::EastAsia,
            "eastasiastage" => LocationName::EastAsiaStage,
            "eastus" => LocationName::EastUS,
            "eastus2" => LocationName::EastUS2,
            "eastus2euap" => LocationName::EastUS2EUAP,
            "eastus2stage" => LocationName::EastUS2Stage,
            "eastusstage" => LocationName::EastUSStage,
            "eastusstg" => LocationName::EastUSSTG,
            "europe" => LocationName::Europe,
            "france" => LocationName::France,
            "francecentral" => LocationName::FranceCentral,
            "francesouth" => LocationName::FranceSouth,
            "germany" => LocationName::Germany,
            "germanynorth" => LocationName::GermanyNorth,
            "germanywestcentral" => LocationName::GermanyWestCentral,
            "global" => LocationName::Global,
            "india" => LocationName::India,
            "indonesia" => LocationName::Indonesia,
            "indonesiacentral" => LocationName::IndonesiaCentral,
            "israel" => LocationName::Israel,
            "israelcentral" => LocationName::IsraelCentral,
            "italy" => LocationName::Italy,
            "italynorth" => LocationName::ItalyNorth,
            "japan" => LocationName::Japan,
            "japaneast" => LocationName::JapanEast,
            "japanwest" => LocationName::JapanWest,
            "jioindiacentral" => LocationName::JioIndiaCentral,
            "jioindiawest" => LocationName::JioIndiaWest,
            "korea" => LocationName::Korea,
            "koreacentral" => LocationName::KoreaCentral,
            "koreasouth" => LocationName::KoreaSouth,
            "malaysia" => LocationName::Malaysia,
            "malaysiawest" => LocationName::MalaysiaWest,
            "mexico" => LocationName::Mexico,
            "mexicocentral" => LocationName::MexicoCentral,
            "newzealand" => LocationName::NewZealand,
            "newzealandnorth" => LocationName::NewZealandNorth,
            "northcentralus" => LocationName::NorthCentralUS,
            "northcentralusstage" => LocationName::NorthCentralUSStage,
            "northeurope" => LocationName::NorthEurope,
            "norway" => LocationName::Norway,
            "norwayeast" => LocationName::NorwayEast,
            "norwaywest" => LocationName::NorwayWest,
            "poland" => LocationName::Poland,
            "polandcentral" => LocationName::PolandCentral,
            "qatar" => LocationName::Qatar,
            "qatarcentral" => LocationName::QatarCentral,
            "singapore" => LocationName::Singapore,
            "southafrica" => LocationName::SouthAfrica,
            "southafricanorth" => LocationName::SouthAfricaNorth,
            "southafricawest" => LocationName::SouthAfricaWest,
            "southcentralus" => LocationName::SouthCentralUS,
            "southcentralusstage" => LocationName::SouthCentralUSStage,
            "southcentralusstg" => LocationName::SouthCentralUSSTG,
            "southeastasia" => LocationName::SoutheastAsia,
            "southeastasiastage" => LocationName::SoutheastAsiaStage,
            "southindia" => LocationName::SouthIndia,
            "spain" => LocationName::Spain,
            "spaincentral" => LocationName::SpainCentral,
            "sweden" => LocationName::Sweden,
            "swedencentral" => LocationName::SwedenCentral,
            "swedensouth" => LocationName::SwedenSouth,
            "switzerland" => LocationName::Switzerland,
            "switzerlandnorth" => LocationName::SwitzerlandNorth,
            "switzerlandwest" => LocationName::SwitzerlandWest,
            "taiwan" => LocationName::Taiwan,
            "uae" => LocationName::UAE,
            "uaecentral" => LocationName::UAECentral,
            "uaenorth" => LocationName::UAENorth,
            "uk" => LocationName::UK,
            "uksouth" => LocationName::UKSouth,
            "ukwest" => LocationName::UKWest,
            "unitedarabemirates" => LocationName::UnitedArabEmirates,
            "unitedkingdom" => LocationName::UnitedKingdom,
            "unitedstates" => LocationName::UnitedStates,
            "unitedstateseuap" => LocationName::UnitedStatesEUAP,
            "westcentralus" => LocationName::WestCentralUS,
            "westeurope" => LocationName::WestEurope,
            "westindia" => LocationName::WestIndia,
            "westus" => LocationName::WestUS,
            "westus2" => LocationName::WestUS2,
            "westus2stage" => LocationName::WestUS2Stage,
            "westus3" => LocationName::WestUS3,
            "westusstage" => LocationName::WestUSStage,
            _ => LocationName::Other(s.to_owned()),
        })
    }
}
impl serde::Serialize for LocationName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for LocationName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::from_str(&value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}

#[cfg(test)]
mod test {
    use crate::location::LocationName;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let locations = [
            "canadacentral",
            "CanadaCentral",
            "CANADACENTRAL",
            "CanadaCENTRAL",
        ];
        let expected = LocationName::CanadaCentral;
        for location in locations {
            let parsed: LocationName = location.parse()?;
            assert_eq!(parsed, expected);
        }

        assert_eq!(expected.to_string(), "canadacentral");
        assert_eq!(serde_json::to_string(&expected)?, "\"canadacentral\"");
        assert_eq!(
            serde_json::from_str::<LocationName>("\"canadaCENTRAL\"")?,
            expected
        );
        Ok(())
    }
}
