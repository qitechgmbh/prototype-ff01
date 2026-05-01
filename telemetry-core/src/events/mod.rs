mod weights;
pub use weights::WeightEvent;

mod plate;
pub use plate::PlateEvent;

mod order;
pub use order::OrderEvent;

mod log;
pub use log::LogEvent;
pub use log::LogCategory;

use crate::EntryDecodeError;

#[derive(Debug)]
pub enum Event {
    Weight(WeightEvent),
    Plate(PlateEvent),
    Order(OrderEvent),
    Log(LogEvent),
}

impl Event {
    const WEIGHT_TAG: u8 = 0;
    const PLATE_TAG:  u8 = 1;
    const ORDER_TAG:  u8 = 2;
    const LOG_TAG:    u8 = 3;

    //TODO: maybe add bounds checks?
    pub fn encode<'a>(&self, out: &'a mut [u8]) -> &'a [u8] {
        let len = match self {
            Event::Weight(event) => {
                out[0] = Self::WEIGHT_TAG;
                event.encode(&mut out[1..]).len()
            }
            Event::Plate(event) => {
                out[0] = Self::PLATE_TAG;
                event.encode(&mut out[1..]).len()
            }
            Event::Order(event) => {
                out[0] = Self::ORDER_TAG;
                event.encode(&mut out[1..]).len()
            }
            Event::Log(event) => {
                out[0] = Self::LOG_TAG;
                event.encode(&mut out[1..]).len()
            }
        };

        &out[0..1 + len]
    }

    pub fn decode(data: &[u8]) -> Result<Self, EntryDecodeError> {
        match data[0] {
            Self::WEIGHT_TAG => {
                let event = WeightEvent::decode(&data[1..])?;
                Ok(Event::Weight(event))
            }
            Self::PLATE_TAG => {
                let event = PlateEvent::decode(&data[1..])?;
                Ok(Event::Plate(event))
            }
            Self::ORDER_TAG => {
                let event = OrderEvent::decode(&data[1..])?;
                Ok(Event::Order(event))
            }
            Self::LOG_TAG => {
                let event = LogEvent::decode(&data[1..])?;
                Ok(Event::Log(event))
            }
            _ => Err(EntryDecodeError::UnknownTag),
        }
    }
}