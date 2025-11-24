
[ ] - MonitorWrapper needs to throw the Connected 
    ContextWrapper::monitor or ContextWrapper::monitor_builder are successfull and the mask for monioring connection exceptions is set

    [ ] - Other execptions also need to be handled properly in MonitorWrapper::try_get_update

[ ] - Line 22 are there any specific exceptions for context_.connect(pv_name_).exec()... such as Connected, Diconnected, Finished etc.?
    If so, we need to catch them specifically and re-throw them as PvxsError with proper messages.

[ ] - Lines 33, MonitorWrapper::stop() is just calling monitor_.reset()
    Need to at least throw the Disconnected exception if the monitor was started before.