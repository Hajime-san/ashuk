import { open } from '@tauri-apps/api/dialog';
import { useState } from 'react';
import { ProcessConnectionError } from '~/libs/error/ProcessConnectionError';
import { PromiseType } from '~/types/util';

export const useOpen = (options?: Parameters<typeof open>[0]) => {
	const [response, setResponse] = useState<PromiseType<ReturnType<typeof open>>>(null);
	const [error, setError] = useState<ProcessConnectionError>();

	const openHandler = async () => {
		try {
			const result = await open(options);
			// reset error
			setError(undefined);
			setResponse(result);
		} catch (err) {
			// reset response
			setResponse(null);
			if (err instanceof ProcessConnectionError) {
				setError(new ProcessConnectionError(err));
				return;
			}
			if (err instanceof Error) {
				setError(new ProcessConnectionError(err));
			}
		}
	};

	return {
		response,
		error,
		openHandler,
		isValidating: !response || !error,
	};
};
