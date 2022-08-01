import { emit } from '@tauri-apps/api/event';
import { useState } from 'react';
import { useIPCQuery } from '~/hooks/useIPCQuery';
import { useOpenDialogQuery } from '~/hooks/useOpenDialogQuery';
import { EmitFileRequestBody } from '../InputFile';
import './style.css';

type CompressOptionsContext = {
	extension: string;
	min: number;
	max: number;
	default: number;
	step: number;
};

const SelectOptions = () => {
	const [options, setOptions] = useState<Array<CompressOptionsContext> | null>(null);
	const [currentOption, setCurrentOption] = useState<CompressOptionsContext | null>(null);
	const request = useIPCQuery<Array<CompressOptionsContext>>({ cmd: 'get_compress_options_context' }, {
		onSuccess: (payload) => {
			const validatedOption = payload.map((v) => {
				return {
					...v,
					// convert step to 0.1
					step: Number.isInteger(v.step) ? v.step : Number(v.step.toFixed(1)),
				};
			});
			setOptions(validatedOption);
			setCurrentOption(validatedOption[0]);
		},
	});

	// update on extension changed
	const onChangeSelectHandler = (e: React.ChangeEvent<HTMLSelectElement>) => {
		if (!options) {
			return;
		}
		const _currentOption = options.find((v) => v.extension === e.target.value)!;
		// update view
		setCurrentOption(_currentOption);

		// update backend
		const requestBody: EmitFileRequestBody = {
			files: null,
			operation: 'Update',
			options: {
				quality: _currentOption.default,
				extension: _currentOption.extension,
			},
		};
		emit('emit-file', requestBody);
	};

	// update on compress option number changed
	const onChangeInputHandler = (e: React.ChangeEvent<HTMLInputElement>) => {
		const requestBody: EmitFileRequestBody = {
			files: null,
			operation: 'Update',
			options: {
				quality: Number(e.target.value),
				extension: currentOption?.extension!,
			},
		};
		emit('emit-file', requestBody);
	};

	return (
		<div className='fixedArea_options'>
			{options
				? (
					<div className='fixedArea_options_box'>
						<select
							name='CompressOptionsContext'
							onChange={onChangeSelectHandler}
							id='CompressOptionsContext'
						>
							{options.map((item) => {
								return <option value={item.extension} key={item.extension}>{item.extension}</option>;
							})}
						</select>
						<input
							type='range'
							name='CompressOptionsContextValue'
							id='CompressOptionsContextValue'
							min={currentOption?.min ? currentOption.min : 0}
							max={currentOption?.max ? currentOption.max : 0}
							step={currentOption?.step ? currentOption.step : 0}
							defaultValue={currentOption?.default ? currentOption.default : 0}
							onChange={onChangeInputHandler}
						/>
					</div>
				)
				: null}
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
		openHandler: () => openRequest.refetch(),
	};
};

export const FixedArea = (props: { compressHandler: () => void }) => {
	const { compressHandler } = props;
	const { openHandler } = useOpenFileDialog();

	const onClearHandler = () => {
		const requestBody: EmitFileRequestBody = {
			files: null,
			operation: 'Clear',
			options: null,
		};
		emit('emit-file', requestBody);
	};

	return (
		<div className='fixedArea'>
			<SelectOptions />
			<div className='fixedArea_buttons'>
				<button
					onClick={onClearHandler}
					className={'fixedArea_clear'}
				>
					Clear list
				</button>
				<button
					onClick={openHandler}
					className={'fixedArea_open'}
				>
					Open file
				</button>
				<button
					onClick={compressHandler}
					className={'fixedArea_start'}
				>
					Compress
				</button>
			</div>
		</div>
	);
};
