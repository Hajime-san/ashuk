import { useCallback, useEffect, useState } from 'react';
import { useOpenDialogQuery } from '~/hooks/useOpenDialogQuery';
import { emit, listen } from '@tauri-apps/api/event';

import './style.css';
import { FixedArea } from '../FixedArea';
import { formatBytes } from '~/libs/util/formatBytes';

import PendingIcon from '@mui/icons-material/Pending';
import AutorenewIcon from '@mui/icons-material/Autorenew';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';

type FileMeta = {
	path: string;
	size: number;
	extension: string;
};

type CompressOptions = {
	quality: number;
	extension: string;
};

export type FileContext = {
	status: 'Initialized' | 'Pending' | 'Success' | 'Failed' | 'Unsupported';
	input: FileMeta;
	output:
		| FileMeta & {
			elapsed: number;
		}
		| null;
};

export type FileListObject = { [key in string]: FileContext };

type Operation = 'Create' | 'Update' | 'Compress' | 'Clear';

export type EmitFileRequestBody = {
	files: FileListObject | null;
	operation: Operation;
	options: CompressOptions | null;
};

const emitFileCreate = async (payload: Array<string>) => {
	const obj: FileListObject = {};
	payload.forEach((key) => {
		const file: FileContext = {
			status: 'Initialized',
			input: {
				path: key,
				size: 0,
				extension: '',
			},
			output: null,
		};
		obj[key] = file;
	});
	const requestBody: EmitFileRequestBody = {
		files: obj,
		operation: 'Create',
		options: null,
	};
	emit('emit-file', requestBody);
};

const useCompress = (files: FileListObject) => {
	const compressHandler = useCallback(() => {
		const requestBody: EmitFileRequestBody = {
			files: files,
			operation: 'Compress',
			options: null,
		};
		emit('emit-file', requestBody);
	}, [files]);

	return {
		compressHandler,
	};
};

const useFileList = () => {
	const request = useOpenDialogQuery(['open_file'], undefined, {
		onSuccess: (payload) => {
			if (!payload || !Array.isArray(payload) || payload.length === 0) {
				return;
			}
			emitFileCreate(payload);
		},
	});
	const [files, setFiles] = useState<FileListObject>({});

	const updateFiles = useCallback((key: string, value: FileContext) => {
		setFiles((obj) => {
			return { ...obj, ...{ [key]: value } };
		});
	}, []);

	useEffect(() => {
		let unlisten: any;
		const f = async () => {
			unlisten = await listen<string>('listen-file', (event) => {
				try {
					const data = JSON.parse(event.payload) as FileContext;
					updateFiles(data.input.path, data);
				} catch (error) {
					console.log(error);
				}
			});
		};
		f();

		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	}, []);

	useEffect(() => {
		let unlisten: any;
		const f = async () => {
			unlisten = await listen<string>('listen-clear-file', (event) => {
				try {
					const data = JSON.parse(event.payload) as Operation;
					if (data === 'Clear') {
						setFiles({});
					}
				} catch (error) {
					console.log(error);
				}
			});
		};
		f();

		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	}, []);

	return {
		files,
	};
};

const FileList = (
	props: { files: FileListObject },
) => {
	const { files } = props;

	return (
		<div className='container'>
			<ul className='tr_h'>
				<li>filename</li>
				<li>size</li>
				<li>compressed</li>
			</ul>
			{Object.entries(files).map(([key, item], i) => {
				const bgColor = i % 2 === 0 ? 'rgb(225 224 224)' : '#fafafa';
				return (
					<ul
						style={{
							backgroundColor: bgColor,
						}}
						className='td_h'
						key={key + String(i)}
					>
						<li>
							<span>{item.input.path}</span>
							<div data-status={item.status}>
								{item.status === 'Initialized' && <PendingIcon sx={{ fill: '#888' }} />}
								{item.status === 'Pending' && <AutorenewIcon />}
								{item.status === 'Success' && <CheckCircleIcon sx={{ fill: '#2e7d32' }} />}
							</div>
						</li>
						<li>{formatBytes(item.input.size)}</li>
						<li>{item.status === 'Success' ? formatBytes(item.output?.size!) : ''}</li>
					</ul>
				);
			})}
		</div>
	);
};

export const InputFile = () => {
	const { files } = useFileList();
	const { compressHandler } = useCompress(files);

	return (
		<div style={{ height: '100%' }}>
			<FileList files={files} />
			<FixedArea compressHandler={compressHandler} />
		</div>
	);
};
