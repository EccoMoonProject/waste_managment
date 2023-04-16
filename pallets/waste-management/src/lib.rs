#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[scale_info(skip_type_params(T))]
	#[derive(Encode, Decode, Clone, PartialEq, Debug, TypeInfo)]
	pub enum WasteStatus {
		Reported,
		Collected,
		Transported,
		Utilized,
	}

	pub type WasteType = u32;
	pub type WasteAmount = u64;
	pub type ReportId = u64;

	#[scale_info(skip_type_params(T))]
	#[derive(Encode, Decode, Clone, PartialEq, Debug, TypeInfo)]
	pub struct WasteData<T: Config> {
		pub report_id: ReportId,
		pub waste_type: WasteType,
		pub waste_amount: WasteAmount,
		pub status: WasteStatus,
		pub location_x: u32,
		pub location_y: u32,
		pub reporter: T::AccountId,
	}

	#[pallet::storage]
	pub(super) type WasteDataCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Maps the WasteData struct to the report_id.
	#[pallet::storage]
	pub(super) type WasteDataMap<T: Config> = StorageMap<_, Twox64Concat, ReportId, WasteData<T>>;

	#[pallet::storage]
	pub(super) type WasteDataByStatus<T: Config> =
		StorageMap<_, Blake2_128Concat, (WasteStatus, ReportId), WasteData<T>>;

	#[pallet::error]
	pub enum Error<T> {
		/// A waste data report must have a unique identifier
		DuplicateReport,
		/// The total number of waste data reports can't exceed the u64 limit
		BoundsOverflow,

		ReportNotFound,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		WasteDataCreated { report_id: ReportId, reporter: T::AccountId },
		WasteStatusUpdated { report_id: ReportId, operator: T::AccountId },
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		#[pallet::call_index(0)]
		pub fn create_waste_data(
			origin: OriginFor<T>,
			waste_type: WasteType,
			waste_amount: WasteAmount,
			location_x: u32,
			location_y: u32,
		) -> DispatchResultWithPostInfo {
			let reporter = ensure_signed(origin)?;

			let report_id =
				WasteDataCount::<T>::get().checked_add(1).ok_or(Error::<T>::BoundsOverflow)?;
			WasteDataCount::<T>::put(report_id);

			let waste_data = WasteData {
				report_id,
				waste_type,
				waste_amount,
				status: WasteStatus::Reported,
				location_x,
				location_y,
				reporter: reporter.clone(),
			};

			WasteDataMap::<T>::try_mutate_exists(report_id, |waste_data_opt| {
				ensure!(waste_data_opt.is_none(), Error::<T>::DuplicateReport);
				*waste_data_opt = Some(waste_data.clone());
				Ok::<(), Error<T>>(())
			})?;

			WasteDataByStatus::<T>::insert((WasteStatus::Reported, report_id), waste_data.clone());

			Self::deposit_event(Event::WasteDataCreated { report_id, reporter });

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		#[pallet::call_index(1)]
		pub fn update_waste_status(
			origin: OriginFor<T>,
			report_id: ReportId,
			new_status: WasteStatus,
		) -> DispatchResultWithPostInfo {
			let operator = ensure_signed(origin)?;
	
			WasteDataMap::<T>::try_mutate(report_id, |waste_data| {
				let waste_data = waste_data.as_mut().ok_or(Error::<T>::ReportNotFound)?;
				let old_status = waste_data.status.clone();
				waste_data.status = new_status.clone();
	
				if old_status != new_status {
					WasteDataByStatus::<T>::remove((old_status, report_id));
					WasteDataByStatus::<T>::insert((new_status, report_id), waste_data.clone());
				}
	
				Ok::<(), Error<T>>(())
			})?;
	
			Self::deposit_event(Event::WasteStatusUpdated { report_id, operator });
	
			Ok(().into())
		}
	}
}