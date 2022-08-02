import { invoke } from '@tauri-apps/api/tauri';
import { ProcessConnectionError } from '~/libs/error/ProcessConnectionError';
import { useQuery, UseQueryOptions } from '@tanstack/react-query';

type IPCParameters = {
	cmd: Parameters<typeof invoke>[0];
	args?: Parameters<typeof invoke>[1];
};

const invokeIPC = async <D = unknown>(params: IPCParameters) => {
	try {
		return await invoke<D>(params.cmd, params.args);
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

export const useIPCQuery = <D = unknown, E = ProcessConnectionError>(
	ipcParams: IPCParameters,
	queryOptions?: UseQueryOptions<D, E>,
) => {
	return useQuery<D, E>([ipcParams.cmd, ipcParams.args], () => invokeIPC<D>(ipcParams), queryOptions);
};
