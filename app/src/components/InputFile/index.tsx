import { useCallback, useEffect, useState } from 'react';
import { useOpen } from '~/hooks/useOpen';
import { valueOf } from '~/types/util';
import { emit, listen } from '@tauri-apps/api/event';

import './style.css';
import { useInternalProcess } from '~/hooks/useInternalProcess';
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

export type EmitFileRequestBody = {
	files: FileListObject | null;
	operation: 'Create' | 'Update' | 'Compress' | 'Delete';
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

const useFileList = (
	openedFiles: valueOf<Pick<ReturnType<typeof useOpen>, 'response'>>,
) => {
	const [files, setFiles] = useState<FileListObject>({});

	const updateFiles = useCallback((key: string, value: FileContext) => {
		setFiles((obj) => {
			return { ...obj, ...{ [key]: value } };
		});
	}, []);

	const openFiles = useCallback(() => {
		if (!openedFiles || openedFiles.length === 0 || !Array.isArray(openedFiles)) {
			return;
		}
		emitFileCreate(openedFiles);
	}, [openedFiles]);

	useEffect(() => {
		openFiles();
	}, [openedFiles]);

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

	return {
		files,
		updateFiles,
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
				const convertedFile = item.status === 'Success';
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
							{convertedFile && <span>✅</span>}
						</li>
						<li>{formatBytes(item.input.size)}</li>
						<li>{convertedFile ? formatBytes(item.output?.size!) : ''}</li>
					</ul>
				);
			})}
		</div>
	);
};

const useOpenFileDialog = () => {
	const request = useInternalProcess<
		Array<{
			ext: string;
			readable: boolean;
			writable: boolean;
		}>
	>('get_supported_extensions');
	const open = useOpen({
		multiple: true,
		filters: request.response
			? [
				{
					name: '*',
					// filter by readble format
					extensions: request.response.filter((v) => v.readable).map((v) => v.ext),
				},
			]
			: [],
	});
	return open;
};

export const InputFile = () => {
	const { response, error, openHandler } = useOpenFileDialog();
	const { files, updateFiles } = useFileList(response);
	const { optimizeHandler } = useOptimize(files);

	return (
		<div style={{ height: '100%' }}>
			<FileList files={files} />
			<FixedArea openHandler={openHandler} optimizeHandler={optimizeHandler} />
		</div>
	);
};
