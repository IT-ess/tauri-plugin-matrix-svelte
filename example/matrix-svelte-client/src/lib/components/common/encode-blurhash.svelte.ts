import { encode } from 'blurhash';

const getImageData = (image: HTMLImageElement) => {
	const canvas = document.createElement('canvas');
	canvas.width = image.width;
	canvas.height = image.height;
	const context = canvas.getContext('2d');
	context?.drawImage(image, 0, 0);
	return context?.getImageData(0, 0, image.width, image.height);
};

export const encodeImageToBlurhash = (imgElement: HTMLImageElement) => {
	const imageData = getImageData(imgElement);
	if (imageData) {
		return encode(imageData.data, imageData.width, imageData.height, 4, 4);
	}
};
