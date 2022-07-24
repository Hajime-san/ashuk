import './style.css';

export const FixedArea = (props: { openHandler: () => Promise<void> }) => {
	const { openHandler } = props
	return (
		<div className='fixedArea'>
			<div className='fixedArea_buttons'>
				<button
				onClick={openHandler}
				className={'fixedArea_open'}
				>Open file</button>
				<button
				onClick={openHandler}
				className={'fixedArea_start'}
				>Optimize</button>
			</div>
		</div>
	);
};
