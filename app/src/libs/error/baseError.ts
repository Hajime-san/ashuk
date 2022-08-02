export class BaseError extends Error {
	constructor(e?: string) {
		super(e);
		Object.defineProperty(this, 'name', {
			configurable: true,
			enumerable: false,
			value: this.constructor.name,
			writable: true,
		});

		if (Error.captureStackTrace) {
			Error.captureStackTrace(this, BaseError);
		}
		Object.setPrototypeOf(this, new.target.prototype);
	}
}
