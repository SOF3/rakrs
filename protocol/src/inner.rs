use rakrs_io::{CanIo, DecodeError, Little, Triad};

#[derive(Debug, Clone)]
pub struct Inner<B: Buffer>
where
    for<'t> &'t B: IntoIterator<Item = &'t u8>,
{
    pub reliability: Reliability,
    pub split: Option<Split>,
    pub buf: B,
}

impl<B: Buffer> Inner<B>
where
    for<'t> &'t B: IntoIterator<Item = &'t u8>,
{
    pub fn write(&self, vec: &mut Vec<u8>) {
        let rel_flag = self.reliability.flag();
        let split_flag = match &self.split {
            Some(_) => 0x10,
            None => 0,
        };
        vec.push(rel_flag.to_u8() | split_flag);

        (self.buf.total_len() as u16 * 8).write(&mut *vec);
        if let Some(reliable) = &self.reliability.reliable {
            reliable.message_index.write(&mut *vec);
        }
        if let Some(order) = &self.reliability.order {
            if let Some(sequence_index) = &order.sequence_index {
                sequence_index.write(&mut *vec);
            }
            order.index.write(&mut *vec);
            order.channel.write(&mut *vec);
        }
        if let Some(split) = &self.split {
            split.write(&mut *vec);
        }
        vec.extend(self.buf.into_iter());
    }
}

impl Inner<Vec<u8>> {
    pub fn read(src: &[u8], offset: &mut usize) -> Result<Self, DecodeError> {
        let flag = u8::read(src, &mut *offset)?;

        let (flag, has_split) = ReliabilityFlag::from_u8(flag)?;
        let Little(Triad(mut buf_len)) = Little::read(src, &mut *offset)?;
        buf_len = buf_len / 8 + (if buf_len % 8 > 0 { 1 } else { 0 });
        if buf_len == 0 {
            return Err(DecodeError::OutOfRange);
        }
        let reliable = match flag.reliable() {
            true => {
                let message_index = Little::<Triad>::read(src, &mut *offset)?;
                Some(Reliable { message_index })
            }
            false => None,
        };
        let order = match flag.ordered() {
            true => {
                let sequence_index = match flag.sequenced() {
                    true => Some(Little::<Triad>::read(src, &mut *offset)?),
                    false => None,
                };
                let index = Little::<Triad>::read(src, &mut *offset)?;
                let channel = u8::read(src, &mut *offset)?;
                Some(Order {
                    index,
                    channel,
                    sequence_index,
                })
            }
            false => None,
        };

        let split = match has_split {
            true => Some(Split::read(src, &mut *offset)?),
            false => None,
        };

        let buf = match src.get((*offset)..(*offset + (buf_len as usize))) {
            Some(buf) => buf,
            None => return Err(DecodeError::UnexpectedEof),
        };
        let buf = buf.to_vec();

        Ok(Self {
            reliability: Reliability {
                reliable,
                with_ack_receipt: flag.with_ack_receipt(),
                order,
            },
            split,
            buf,
        })
    }
}

pub trait Buffer
where
    for<'t> &'t Self: IntoIterator<Item = &'t u8>,
{
    fn total_len(&self) -> usize;
}

impl Buffer for Vec<u8> {
    fn total_len(&self) -> usize {
        self.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReliabilityFlag {
    Unreliable,
    UnreliableSequenced,
    Reliable,
    ReliableOrdered,
    ReliableSequenced,
    UnreliableWithAckReceipt,
    ReliableWithAckReceipt,
    ReliableOrderedWithAckReceipt,
}

impl ReliabilityFlag {
    fn from_u8(flag: u8) -> Result<(Self, bool), DecodeError> {
        use ReliabilityFlag::*;
        Ok((
            match (flag >> 5) & 7 {
                0 => Unreliable,
                1 => UnreliableSequenced,
                2 => Reliable,
                3 => ReliableOrdered,
                4 => ReliableSequenced,
                5 => UnreliableWithAckReceipt,
                6 => ReliableWithAckReceipt,
                7 => ReliableOrderedWithAckReceipt,
                _ => return Err(DecodeError::OutOfRange),
            },
            (flag & 0x10) > 0,
        ))
    }

    fn to_u8(self) -> u8 {
        let flag = match self {
            Self::Unreliable => 0,
            Self::UnreliableSequenced => 1,
            Self::Reliable => 2,
            Self::ReliableOrdered => 3,
            Self::ReliableSequenced => 4,
            Self::UnreliableWithAckReceipt => 5,
            Self::ReliableWithAckReceipt => 6,
            Self::ReliableOrderedWithAckReceipt => 7,
        };
        flag << 5
    }

    fn reliable(self) -> bool {
        use ReliabilityFlag::*;
        matches!(
            self,
            Reliable
                | ReliableOrdered
                | ReliableSequenced
                | ReliableWithAckReceipt
                | ReliableOrderedWithAckReceipt
        )
    }

    fn ordered(self) -> bool {
        use ReliabilityFlag::*;
        matches!(
            self,
            UnreliableSequenced
                | ReliableOrdered
                | ReliableSequenced
                | ReliableOrderedWithAckReceipt
        )
    }

    fn sequenced(self) -> bool {
        use ReliabilityFlag::*;
        matches!(self, UnreliableSequenced | ReliableSequenced)
    }

    fn with_ack_receipt(self) -> bool {
        use ReliabilityFlag::*;
        matches!(
            self,
            UnreliableWithAckReceipt | ReliableWithAckReceipt | ReliableOrderedWithAckReceipt
        )
    }
}

#[derive(Debug, Clone)]
pub struct Reliability {
    pub reliable: Option<Reliable>,
    pub with_ack_receipt: bool,
    pub order: Option<Order>,
}

impl Reliability {
    fn flag(&self) -> ReliabilityFlag {
        use ReliabilityFlag::*;
        match self.reliable {
            Some(_) => match &self.order {
                Some(order) => match order.sequence_index {
                    Some(_) => ReliableSequenced,
                    None if self.with_ack_receipt => ReliableOrderedWithAckReceipt,
                    None => ReliableOrdered,
                },
                None if self.with_ack_receipt => ReliableWithAckReceipt,
                None => Reliable,
            },
            None => {
                match &self.order {
                    Some(order) if order.sequence_index.is_some() => UnreliableSequenced,
                    Some(order) => {
                        log::warn!("Unreliable ordered unsequenced packet is coerced to unreliable unordered");
                        Unreliable // should this panick instead?
                    }
                    None if self.with_ack_receipt => UnreliableWithAckReceipt,
                    None => Unreliable,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Reliable {
    pub message_index: Little<Triad>,
}

#[derive(Debug, Clone)]
pub struct Order {
    index: Little<Triad>,
    channel: u8,
    sequence_index: Option<Little<Triad>>,
}

can_io! {
    struct Split {
        pub count: u32,
        pub id: u16,
        pub index: u32,
    }
}
