import { emit } from '@tauri-apps/api/event';
import { useCallback, useEffect, useState } from 'react';
import { useInternalProcess } from '~/hooks/useInternalProcess';
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
	const { response } = useInternalProcess<Array<CompressOptionsContext>>('get_compress_options_context');
	const [currentOption, setCurrentOption] = useState<CompressOptionsContext | null>(null);

	useEffect(() => {
		if (!response) {
			return;
		}
		setCurrentOption(response[0]);
	}, [response]);

	const onChangeSelectHandler = (e: React.ChangeEvent<HTMLSelectElement>) => {
		if (!response) {
			return;
		}
		const _currentOption = response.find((v) => v.extension === e.target.value)!;
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
			{response
				? (
					<div className='fixedArea_options_box'>
						<select
							name='CompressOptionsContext'
							onChange={onChangeSelectHandler}
							id='CompressOptionsContext'
						>
							{response.map((item) => {
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

export const FixedArea = (props: { openHandler: () => Promise<void>; optimizeHandler: () => void }) => {
	const { openHandler, optimizeHandler } = props;

	const onDeleteHandler = () => {
		const requestBody: EmitFileRequestBody = {
			files: null,
			operation: 'Delete',
			options: null,
		};
		emit('emit-file', requestBody);
	};
	return (
		<div className='fixedArea'>
			<SelectOptions />
			<div className='fixedArea_buttons'>
				<button
					onClick={onDeleteHandler}
					className={'fixedArea_delete'}
				>
					Delete File
				</button>
				<button
					onClick={openHandler}
					className={'fixedArea_open'}
				>
					Open file
				</button>
				<button
					onClick={optimizeHandler}
					className={'fixedArea_start'}
				>
					Optimize
				</button>
			</div>
		</div>
	);
};
