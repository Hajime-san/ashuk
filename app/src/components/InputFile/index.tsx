import { useEffect } from 'react';
import { useMutationInternalProcess } from '~/hooks/useInternalProcess';
import { useOpen } from '~/hooks/useOpen';
import { valueOf } from '~/types/util';

type ImageContext = {
	path: string;
	size: number;
};

type ConvertResult = {
	status: 'Success' | 'Failed';
	elapsed: {
		nano_sec: number;
		sec: number;
	};
	input: ImageContext;
	output: ImageContext;
};

const useConvertFiles = () => {
	return useMutationInternalProcess<Array<ConvertResult>, { filePath: Array<string> }>(
		'command_covert_to_webp',
	);
};

const formatBytes = (bytes: number, decimals = 2) => {
	if (bytes === 0) return '0 Bytes';

	const k = 1024;
	const dm = decimals < 0 ? 0 : decimals;
	const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

	const i = Math.floor(Math.log(bytes) / Math.log(k));

	return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
};

const FileList = (
	props: { openedFiles: valueOf<Pick<ReturnType<typeof useOpen>, 'response'>> },
) => {
	const { openedFiles } = props;
	const convertedProcess = useConvertFiles();

	useEffect(() => {
		if (!openedFiles) {
			return;
		}
		convertedProcess.mutate({ filePath: openedFiles as Array<string> });
	}, [openedFiles]);

	return (
		<div style={{ overflowY: 'scroll', height: 'calc(100% - (62px + 1rem))' }}>
			<ul
				style={{
					position: 'sticky',
					top: 0,
					left: 0,
					display: 'grid',
					gridTemplateColumns: '2fr 1fr 1fr',
					columnGap: '1rem',
					justifyContent: 'space-between',
				}}
			>
				<li>filename</li>
				<li>size(kb)</li>
				<li>optimized size(kb)</li>
			</ul>
			{Array.isArray(openedFiles) &&
				openedFiles.map((item, i) => {
					const convertedFile = convertedProcess.response?.find((v) => v.input.path === item);
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
							key={item + String(i)}
						>
							<li>{item}</li>
							<li>{convertedFile && formatBytes(convertedFile?.input.size)}</li>
							<li>{convertedFile && formatBytes(convertedFile?.output.size)}</li>
						</ul>
					);
				})}
		</div>
	);
};

export const InputFile = () => {
	const { response, error, openHandler } = useOpen({
		multiple: true,
		// filters: [{
		//     name: '.*',
		//     extensions: ['png', 'jpeg']
		// }]
	});

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
				<input id='file' accept='image/png, image/jpeg' hidden />
			</div>
		</div>
	);
};
