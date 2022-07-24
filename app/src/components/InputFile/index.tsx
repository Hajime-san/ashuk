import { useCallback, useEffect, useState } from 'react';
import { useOpen } from '~/hooks/useOpen';
import { valueOf } from '~/types/util';
import { emit, listen } from '@tauri-apps/api/event';

import './style.css';
import { useInternalProcess } from '~/hooks/useInternalProcess';

type FileMeta = {
	path: string;
	size: number;
};

type FileContext = {
	id: number;
	status: 'Initialized' | 'Pending' | 'Success' | 'Failed' | 'Unsupported';
	input: FileMeta;
	output: FileMeta & {
		elapsed: number;
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
	const [files, setFiles] = useState<Map<number, FileContext>>(new Map());

	const updateFiles = useCallback((key: number, value: FileContext) => {
		setFiles((map) => new Map(map.set(key, value)));
	}, []);

	useEffect(() => {
		if (!openedFiles || openedFiles.length === 0) {
			return;
		}
		emit('emit-file', openedFiles);
	}, [openedFiles]);

	useEffect(() => {
		let unlisten: any;
		const f = async () => {
			unlisten = await listen<string>('listen-file', (event) => {
				try {
					const data = JSON.parse(event.payload) as FileContext;
					updateFiles(data.id, data);
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
	props: { openedFiles: valueOf<Pick<ReturnType<typeof useOpen>, 'response'>> },
) => {
	const { files } = useFileList(props.openedFiles);

	return (
		<div style={{ overflowY: 'scroll', height: 'calc(100vh - (1rem * 2) - (62px + 1rem))' }}>
			<ul className='th'>
				<li>filename</li>
				<li>size(kb)</li>
				<li>optimized size(kb)</li>
			</ul>
			{[...files].map(([id, item], i) => {
				const convertedFile = item.status === 'Success';
				const bgColor = i % 2 === 0 ? 'rgb(225 224 224)' : '#fafafa';
				return (
					<ul
						style={{
							display: 'grid',
							gridTemplateColumns: '2fr 1fr 1fr',
							columnGap: '1rem',
							justifyContent: 'space-between',
							padding: '0.2rem',
							backgroundColor: bgColor,
						}}
						key={id + String(i)}
					>
						<li>
							<span>{item.input.path}</span>
							{convertedFile && <span>✅</span>}
						</li>
						<li>{formatBytes(item.input.size)}</li>
						<li>{convertedFile && formatBytes(item.output.size)}</li>
					</ul>
				);
			})}
		</div>
	);
};

const useOpenFileDialog = () => {
	const request = useInternalProcess<Array<{
		ext: string,
		readable: boolean,
		writable: boolean
	}>>('get_supported_extentions')
	const open = useOpen({
		multiple: true,
		filters: request.response ? [
			{
			    name: '*',
				// filter by readble format
			    extensions: request.response.filter(v => v.readable).map(v => v.ext)
			}
		] : []
	});
	return open
}

export const InputFile = () => {
	const { response, error, openHandler } = useOpenFileDialog()

	return (
		<div style={{ height: '100%' }}>
			<FileList openedFiles={response} />
			<div style={{ position: 'fixed', right: 0, bottom: 0, margin: '1rem' }}>
				<label
					htmlFor='file'
					onClick={openHandler}
					style={{
						padding: '1rem',
						backgroundColor: '#00c3ff',
						width: 200,
						height: 30,
						display: 'inline-flex',
						alignItems: 'center',
						borderRadius: '10px',
						cursor: 'pointer',
					}}
				>
					<p style={{ width: '100%', textAlign: 'center', color: '#fff' }}>Open file</p>
				</label>
				<input id='file' hidden />
			</div>
		</div>
	);
};
