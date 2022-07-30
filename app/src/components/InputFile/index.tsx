import { useCallback, useEffect, useState } from 'react';
import { useOpenDialogQuery } from '~/hooks/useOpenDialogQuery';
import { emit, listen } from '@tauri-apps/api/event';

import './style.css';
import { useIPCQuery } from '~/hooks/useIPCQuery';
import { FixedArea } from '../FixedArea';

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
	input: FileMeta & {
		writable_extensions: Array<string>;
	};
	output:
		| FileMeta & {
			elapsed: number;
		}
		| null;
};

export type FileListObject = { [key in string]: FileContext };

type Operation = 'Create' | 'Update' | 'Compress' | 'Delete';

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
				writable_extensions: [],
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

const emitFileUpdate = async (payload: FileListObject) => {
	const requestBody: EmitFileRequestBody = {
		files: payload,
		operation: 'Compress',
		options: null,
	};
	emit('emit-file', requestBody);
};

const useOptimize = (files: FileListObject) => {
	const optimizeHandler = useCallback(() => {
		emitFileUpdate(files);
	}, [files]);

	return {
		optimizeHandler,
	};
};

const formatBytes = (bytes: number, decimals = 2) => {
	if (bytes === 0) return '0 Bytes';

	const k = 1024;
	const dm = decimals < 0 ? 0 : decimals;
	const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

	const i = Math.floor(Math.log(bytes) / Math.log(k));

	return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
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
			unlisten = await listen<string>('listen-delete-file', (event) => {
				try {
					const data = JSON.parse(event.payload) as Operation;
					if (data === 'Delete') {
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
			<ul className='th'>
				<li>filename</li>
				<li>size</li>
				<li>optimized</li>
			</ul>
			{Object.entries(files).map(([key, item], i) => {
				const compressedFile = item.status === 'Success';
				const bgColor = i % 2 === 0 ? 'rgb(225 224 224)' : '#fafafa';
				return (
					<ul
						style={{
							display: 'grid',
							gridTemplateColumns: '3fr 1fr 1fr',
							columnGap: '1rem',
							justifyContent: 'space-between',
							padding: '0.2rem',
							backgroundColor: bgColor,
						}}
						key={key + String(i)}
					>
						<li>
							<span>{item.input.path}</span>
							{compressedFile && <span>✅</span>}
						</li>
						<li>{formatBytes(item.input.size)}</li>
						<li>{compressedFile ? formatBytes(item.output?.size!) : ''}</li>
					</ul>
				);
			})}
		</div>
	);
};

const useOpenFileDialog = () => {
	// get file filter extensions
	const request = useIPCQuery<
		Array<{
			ext: string;
			readable: boolean;
			writable: boolean;
		}>
	>({ cmd: 'get_supported_extensions' });

	const openRequest = useOpenDialogQuery(
		['open_file'],
		{
			multiple: true,
			filters: request.data
				? [
					{
						name: '*',
						// filter by readble format
						extensions: request.data.filter((v) => v.readable).map((v) => v.ext),
					},
				]
				: [],
		},
	);

	return {
		openHandler: openRequest.refetch,
	};
};

export const InputFile = () => {
	const { openHandler } = useOpenFileDialog();
	const { files } = useFileList();
	const { optimizeHandler } = useOptimize(files);

	return (
		<div style={{ height: '100%' }}>
			<FileList files={files} />
			<FixedArea openHandler={openHandler as any} optimizeHandler={optimizeHandler} />
		</div>
	);
};
