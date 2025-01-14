use frame_support::BoundedVec;

pub trait MetadataValidator<CollectionId, ItemId, StringLimit> {
	fn validate_metadata(
		collection: CollectionId,
		item: ItemId,
		metadata: &BoundedVec<u8, StringLimit>,
	) -> bool;
}

impl<CollectionId, ItemId, StringLimit> MetadataValidator<CollectionId, ItemId, StringLimit>
	for ()
{
	fn validate_metadata(_: CollectionId, _: ItemId, _: &BoundedVec<u8, StringLimit>) -> bool {
		true
	}
}
