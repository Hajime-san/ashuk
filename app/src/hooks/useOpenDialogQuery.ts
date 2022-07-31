import { open } from '@tauri-apps/api/dialog';
import { ProcessConnectionError } from '~/libs/error/ProcessConnectionError';
import { QueryKey, useQuery, UseQueryOptions } from '@tanstack/react-query';

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
