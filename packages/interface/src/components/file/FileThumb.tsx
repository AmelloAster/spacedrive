import { useBridgeQuery } from '@sd/client';
import { FilePath } from '@sd/core';
import clsx from 'clsx';
import React, { useContext } from 'react';

import { AppPropsContext } from '../../AppPropsContext';
import icons from '../../assets/icons';
import { Folder } from '../icons/Folder';

export default function FileThumb(props: {
	file: FilePath;
	locationId: number;
	hasThumbnailOverride: boolean;
	className?: string;
}) {
	const appProps = useContext(AppPropsContext);

	if (props.file.is_dir) {
		return <Folder size={100} />;
	}
	if (props.file.file?.has_thumbnail || props.hasThumbnailOverride) {
		return (
			<img
				className="pointer-events-none z-90"
				src={appProps?.getThumbnailUrlById(props.file.file!.cas_id)}
			/>
		);
	}

	if (icons[props.file.extension as keyof typeof icons]) {
		let Icon = icons[props.file.extension as keyof typeof icons];
		return <Icon className={clsx('max-w-[170px] w-full h-full', props.className)} />;
	}
	return <div></div>;
}
