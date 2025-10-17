use compact_str::CompactString;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LocationName {
    EastUS,
    WestUS2,
    AustraliaEast,
    SoutheastAsia,
    NorthEurope,
    SwedenCentral,
    SwedenSouth,
    UKSouth,
    WestEurope,
    CentralUS,
    SouthAfricaNorth,
    CentralIndia,
    EastAsia,
    IndonesiaCentral,
    JapanEast,
    JapanWest,
    KoreaCentral,
    MalaysiaWest,
    NewZealandNorth,
    CanadaCentral,
    AustriaEast,
    FranceCentral,
    GermanyWestCentral,
    ItalyNorth,
    NorwayEast,
    PolandCentral,
    SpainCentral,
    SwitzerlandNorth,
    MexicoCentral,
    UAENorth,
    BrazilSouth,
    ChileCentral,
    EastUS2EUAP,
    IsraelCentral,
    QatarCentral,
    CentralUSStage,
    EastUSStage,
    EastUS2Stage,
    NorthCentralUSStage,
    SouthCentralUSStage,
    WestUSStage,
    WestUS2Stage,
    Asia,
    AsiaPacific,
    Australia,
    Brazil,
    Canada,
    Europe,
    France,
    Germany,
    Global,
    India,
    Indonesia,
    Israel,
    Italy,
    Japan,
    Korea,
    Malaysia,
    Mexico,
    NewZealand,
    Norway,
    Poland,
    Qatar,
    Singapore,
    SouthAfrica,
    Spain,
    Sweden,
    Switzerland,
    Taiwan,
    UnitedArabEmirates,
    UnitedKingdom,
    UnitedStates,
    UnitedStatesEUAP,
    EastAsiaStage,
    SoutheastAsiaStage,
    BrazilUS,
    EastUS2,
    EastUSSTG,
    SouthCentralUS,
    WestUS3,
    NorthCentralUS,
    WestUS,
    JioIndiaWest,
    CentralUSEUAP,
    SouthCentralUSSTG,
    WestCentralUS,
    SouthAfricaWest,
    AustraliaCentral,
    AustraliaCentral2,
    AustraliaSoutheast,
    JioIndiaCentral,
    KoreaSouth,
    SouthIndia,
    WestIndia,
    CanadaEast,
    FranceSouth,
    GermanyNorth,
    NorwayWest,
    SwitzerlandWest,
    UKWest,
    UAECentral,
    BrazilSoutheast,
    UK,
    UAE,
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
            "eastus" => LocationName::EastUS,
            "westus2" => LocationName::WestUS2,
            "australiaeast" => LocationName::AustraliaEast,
            "southeastasia" => LocationName::SoutheastAsia,
            "northeurope" => LocationName::NorthEurope,
            "swedencentral" => LocationName::SwedenCentral,
            "swedensouth" => LocationName::SwedenSouth,
            "uksouth" => LocationName::UKSouth,
            "westeurope" => LocationName::WestEurope,
            "centralus" => LocationName::CentralUS,
            "southafricanorth" => LocationName::SouthAfricaNorth,
            "centralindia" => LocationName::CentralIndia,
            "eastasia" => LocationName::EastAsia,
            "indonesiacentral" => LocationName::IndonesiaCentral,
            "japaneast" => LocationName::JapanEast,
            "japanwest" => LocationName::JapanWest,
            "koreacentral" => LocationName::KoreaCentral,
            "malaysiawest" => LocationName::MalaysiaWest,
            "newzealandnorth" => LocationName::NewZealandNorth,
            "canadacentral" => LocationName::CanadaCentral,
            "austriaeast" => LocationName::AustriaEast,
            "francecentral" => LocationName::FranceCentral,
            "germanywestcentral" => LocationName::GermanyWestCentral,
            "italynorth" => LocationName::ItalyNorth,
            "norwayeast" => LocationName::NorwayEast,
            "polandcentral" => LocationName::PolandCentral,
            "spaincentral" => LocationName::SpainCentral,
            "switzerlandnorth" => LocationName::SwitzerlandNorth,
            "mexicocentral" => LocationName::MexicoCentral,
            "uaenorth" => LocationName::UAENorth,
            "brazilsouth" => LocationName::BrazilSouth,
            "chilecentral" => LocationName::ChileCentral,
            "eastus2euap" => LocationName::EastUS2EUAP,
            "israelcentral" => LocationName::IsraelCentral,
            "qatarcentral" => LocationName::QatarCentral,
            "centralusstage" => LocationName::CentralUSStage,
            "eastusstage" => LocationName::EastUSStage,
            "eastus2stage" => LocationName::EastUS2Stage,
            "northcentralusstage" => LocationName::NorthCentralUSStage,
            "southcentralusstage" => LocationName::SouthCentralUSStage,
            "westusstage" => LocationName::WestUSStage,
            "westus2stage" => LocationName::WestUS2Stage,
            "asia" => LocationName::Asia,
            "asiapacific" => LocationName::AsiaPacific,
            "australia" => LocationName::Australia,
            "brazil" => LocationName::Brazil,
            "canada" => LocationName::Canada,
            "europe" => LocationName::Europe,
            "france" => LocationName::France,
            "germany" => LocationName::Germany,
            "global" => LocationName::Global,
            "india" => LocationName::India,
            "indonesia" => LocationName::Indonesia,
            "israel" => LocationName::Israel,
            "italy" => LocationName::Italy,
            "japan" => LocationName::Japan,
            "korea" => LocationName::Korea,
            "malaysia" => LocationName::Malaysia,
            "mexico" => LocationName::Mexico,
            "newzealand" => LocationName::NewZealand,
            "norway" => LocationName::Norway,
            "poland" => LocationName::Poland,
            "qatar" => LocationName::Qatar,
            "singapore" => LocationName::Singapore,
            "southafrica" => LocationName::SouthAfrica,
            "spain" => LocationName::Spain,
            "sweden" => LocationName::Sweden,
            "switzerland" => LocationName::Switzerland,
            "taiwan" => LocationName::Taiwan,
            "unitedarabemirates" => LocationName::UnitedArabEmirates,
            "unitedkingdom" => LocationName::UnitedKingdom,
            "unitedstates" => LocationName::UnitedStates,
            "unitedstateseuap" => LocationName::UnitedStatesEUAP,
            "eastasiastage" => LocationName::EastAsiaStage,
            "southeastasiastage" => LocationName::SoutheastAsiaStage,
            "brazilus" => LocationName::BrazilUS,
            "eastus2" => LocationName::EastUS2,
            "eastusstg" => LocationName::EastUSSTG,
            "southcentralus" => LocationName::SouthCentralUS,
            "westus3" => LocationName::WestUS3,
            "northcentralus" => LocationName::NorthCentralUS,
            "westus" => LocationName::WestUS,
            "jioindiawest" => LocationName::JioIndiaWest,
            "centraluseuap" => LocationName::CentralUSEUAP,
            "southcentralusstg" => LocationName::SouthCentralUSSTG,
            "westcentralus" => LocationName::WestCentralUS,
            "southafricawest" => LocationName::SouthAfricaWest,
            "australiacentral" => LocationName::AustraliaCentral,
            "australiacentral2" => LocationName::AustraliaCentral2,
            "australiasoutheast" => LocationName::AustraliaSoutheast,
            "jioindiacentral" => LocationName::JioIndiaCentral,
            "koreasouth" => LocationName::KoreaSouth,
            "southindia" => LocationName::SouthIndia,
            "westindia" => LocationName::WestIndia,
            "canadaeast" => LocationName::CanadaEast,
            "francesouth" => LocationName::FranceSouth,
            "germanynorth" => LocationName::GermanyNorth,
            "norwaywest" => LocationName::NorwayWest,
            "switzerlandwest" => LocationName::SwitzerlandWest,
            "ukwest" => LocationName::UKWest,
            "uaecentral" => LocationName::UAECentral,
            "brazilsoutheast" => LocationName::BrazilSoutheast,
            "uk" => LocationName::UK,
            "uae" => LocationName::UAE,
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
