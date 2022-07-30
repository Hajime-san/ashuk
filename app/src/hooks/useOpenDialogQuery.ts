import { open } from '@tauri-apps/api/dialog';
import { useState } from 'react';
import { ProcessConnectionError } from '~/libs/error/ProcessConnectionError';
import { PromiseType } from '~/types/util';
import { QueryKey, useQuery, UseQueryOptions } from '@tanstack/react-query';

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

type openParameters = Parameters<typeof open>[0];

const openDialog = async (options?: openParameters) => {
	try {
		return await open(options);
	} catch (err) {
		if (err instanceof ProcessConnectionError) {
			throw new ProcessConnectionError(err);
		}
		if (err instanceof Error) {
			throw new ProcessConnectionError(err);
		}
		throw err;
	}
};

export const useOpenDialogQuery = <E = ProcessConnectionError>(
	queryKey: QueryKey,
	options?: openParameters,
	queryOptions?: UseQueryOptions<ReturnType<typeof openDialog>, E>,
) => {
	return useQuery<ReturnType<typeof openDialog>, E>(queryKey, () => openDialog(options), {
		...queryOptions,
		// use refetch on event
		enabled: false,
	});
};
