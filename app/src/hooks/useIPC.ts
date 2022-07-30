import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { ProcessConnectionError } from '~/libs/error/ProcessConnectionError';

export const useIPC = <T = unknown>(
	cmd: Parameters<typeof invoke>[0],
	args?: Parameters<typeof invoke>[1],
) => {
	const [response, setResponse] = useState<T>();
	const [error, setError] = useState<ProcessConnectionError>();

	useEffect(() => {
		const f = async () => {
			try {
				const result = await invoke<T>(cmd, args);
				// reset error
				setError(undefined);
				setResponse(result);
			} catch (err) {
				// reset response
				setResponse(undefined);
				if (err instanceof ProcessConnectionError) {
					setError(new ProcessConnectionError(err));
					return;
				}
				if (err instanceof Error) {
					setError(new ProcessConnectionError(err));
				}
			}
		};
		f();
	}, []);

	return {
		response,
		error,
		isValidating: !response || !error,
	};
};
