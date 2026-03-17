use super::{Quote, QuoteDto};
use crate::domain::DomainError;

impl TryFrom<QuoteDto> for Quote {
    type Error = DomainError;

    fn try_from(value: QuoteDto) -> Result<Self, Self::Error> {
        Quote::new(
            value.id,
            value.inline,
            value.external,
            value.markdown,
            value.image,
            value.remark,
        )
    }
}

impl From<Quote> for QuoteDto {
    fn from(value: Quote) -> Self {
        Self {
            id: value.id,
            inline: value.inline,
            external: value.external,
            markdown: value.markdown,
            image: value.image,
            remark: value.remark,
        }
    }
}
