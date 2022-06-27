import { DotsVerticalIcon, RefreshIcon } from '@heroicons/react/outline';
import { CogIcon, TrashIcon } from '@heroicons/react/solid';
import { command, useBridgeCommand } from '@sd/client';
import { LocationResource } from '@sd/core';
import { Button } from '@sd/ui';
import clsx from 'clsx';
import React, { useState } from 'react';

import { Folder } from '../icons/Folder';
import Card from '../layout/Card';
import Dialog from '../layout/Dialog';

interface LocationListItemProps {
	location: LocationResource;
}

export default function LocationListItem({ location }: LocationListItemProps) {
	const [showDeleteLocModal, setShowDeleteLocModal] = useState(false);

	const { mutate: locRescan } = useBridgeCommand('LocRescan');

	const { mutate: deleteLoc, isLoading: locDeletePending } = useBridgeCommand('LocDelete', {
		onSuccess: () => {
			setShowDeleteLocModal(false);
		}
	});

	return (
		<Card>
			<DotsVerticalIcon className="w-5 h-5 mt-3 mr-1 -ml-3 cursor-move drag-handle opacity-10" />
			<Folder size={30} className="mr-3" />
			<div className="flex flex-col">
				<h1 className="pt-0.5 text-sm font-semibold">{location.name}</h1>
				<p className="mt-0.5 text-sm select-text text-gray-250">
					<span className="py-[1px] px-1 bg-gray-500 rounded mr-1">{location.node?.name}</span>
					{location.path}
				</p>
			</div>
			<div className="flex flex-grow" />
			<div className="flex h-[45px] p-2 space-x-2">
				<Button disabled variant="gray" className="!py-1.5 !px-2.5 pointer-events-none flex">
					<>
						<div
							className={clsx(
								'w-2 h-2  rounded-full',
								location.is_online ? 'bg-green-500' : 'bg-red-500'
							)}
						/>
					</>
				</Button>
				<Dialog
					open={showDeleteLocModal}
					onOpenChange={setShowDeleteLocModal}
					title="Delete Location"
					description="Deleting a location will also remove all files associated with it from the Spacedrive database, the files themselves will not be deleted."
					ctaAction={() => {
						deleteLoc({ id: location.id });
					}}
					loading={locDeletePending}
					ctaDanger
					ctaLabel="Delete"
					trigger={
						<Button variant="gray" className="!p-1.5">
							<TrashIcon className="w-4 h-4" />
						</Button>
					}
				/>
				<Button
					variant="gray"
					className="!p-1.5"
					onClick={() => {
						// this should cause a lite directory rescan, but this will do for now, so the button does something useful
						locRescan({ id: location.id });
					}}
				>
					<RefreshIcon className="w-4 h-4" />
				</Button>
				{/* <Button variant="gray" className="!p-1.5">
					<CogIcon className="w-4 h-4" />
				</Button> */}
			</div>
		</Card>
	);
}
