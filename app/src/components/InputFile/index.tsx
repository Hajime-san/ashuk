import { useEffect } from 'react'
import { useMutationInternalProcess } from '~/hooks/useInternalProcess'
import { useOpen } from '~/hooks/useOpen'

export const InputFile = () => {
    const { response, error, openHandler } = useOpen({
        multiple: true,
        // filters: [{
        //     name: '.*',
        //     extensions: ['png', 'jpeg']
        // }]
    })

    const convert = useMutationInternalProcess<string, { filePath: Array<string> }>('command_covert_to_webp')

    useEffect(() => {
        if(!response) {
            return
        }
        console.log(response);
        convert.mutate({ filePath: response as Array<string> })
    }, [response])

    useEffect(() => {
        console.log(convert.response);
        console.log(convert.error);
    }, [convert])

  return (
    <div>
        <label htmlFor="file" onClick={openHandler} style={{ padding: 10, backgroundColor: '#00c3ff', width: 400, height: 50, display: 'inline-flex', borderRadius: '10px', cursor: 'pointer' }}>
            <p style={{ width: '100%', textAlign: 'center', color: '#fff' }}>Click here</p>
        </label>
        <input id='file' accept="image/png, image/jpeg" hidden/>
        <p>filename: {response}</p>
    </div>
  )
}
