import { BaseError } from '~/libs/error/baseError';
import { SerializedErrorBaseObject, SerializedErrorObject } from '~/libs/error/SerializedErrorObject';

export class ProcessConnectionError extends BaseError {
	context: SerializedErrorBaseObject;
	constructor(context: SerializedErrorBaseObject, e?: string) {
		super(e);
		this.name = 'ProcessConnectionError';
		this.context = context;
	}
	serialize(): SerializedErrorObject {
		return {
			name: this.name,
			message: this.context.message,
			status: this.context.status,
		};
	}
}
