var IndexManager = (function() {
  return {
    _REFRESH_TIMEOUT : 5000,
    _REFRESH_INTERVAL : 20000,

    _SELECTOR_ASIDE : null,
    _SELECTOR_MAIN : null,

    bind : function() {
      IndexManager._SELECTOR_ASIDE = (
        (document.getElementsByTagName("aside") || [])[0]
      );
      IndexManager._SELECTOR_MAIN = (
        (document.getElementsByTagName("main") || [])[0]
      );

      if (IndexManager._SELECTOR_ASIDE && IndexManager._SELECTOR_MAIN) {
        IndexManager.__schedule_refresh();
      }
    },

    __schedule_refresh : function() {
      setTimeout(function() {
        IndexManager.__load(
          "/status/text/", "text",

          IndexManager.__handle_status_text_done_from_request,
          IndexManager.__handle_status_text_error
        );
      }, IndexManager._REFRESH_INTERVAL);
    },

    __handle_status_text_done_from_request : function(request) {
      IndexManager.__handle_status_text_done(
        request.responseText || window.STATUS_GENERAL
      );
    },

    __handle_status_text_done : function(status) {
      if (status !== window.STATUS_GENERAL) {
        window.STATUS_GENERAL = status;

        IndexManager.__load(
          "/", "document",

          IndexManager.__handle_base_done,
          IndexManager.__handle_base_error
        );
      } else {
        IndexManager.__schedule_refresh();
      }
    },

    __handle_status_text_error : function() {
      IndexManager.__handle_status_text_done(
        window.STATUS_GENERAL
      );
    },

    __handle_base_done : function(request) {
      if (request && request.response && request.response.body) {
        var aside_sel = (
          request.response.body.getElementsByTagName("aside") || []
        )[0];
        var main_sel = (
          request.response.body.getElementsByTagName("main") || []
        )[0];

        if (aside_sel && main_sel) {
          IndexManager._SELECTOR_ASIDE.parentNode.replaceChild(
            aside_sel, IndexManager._SELECTOR_ASIDE
          );
          IndexManager._SELECTOR_MAIN.parentNode.replaceChild(
            main_sel, IndexManager._SELECTOR_MAIN
          );

          IndexManager._SELECTOR_ASIDE = aside_sel;
          IndexManager._SELECTOR_MAIN = main_sel;
        }
      }

      IndexManager.__schedule_refresh();
    },

    __handle_base_error : function() {
      IndexManager.__handle_base_done(null);
    },

    __load : function(path, type, fn_handle_done, fn_handle_error) {
      var request = new XMLHttpRequest();

      request.open("GET", path, true);

      request.responseType = type;
      request.timeout = IndexManager._REFRESH_TIMEOUT;

      request.onreadystatechange = function() {
        // Request finished.
        if (request.readyState === 4) {
          if (request.status === 200) {
            if (typeof fn_handle_done === "function") {
              fn_handle_done(request);
            }
          } else {
            if (typeof fn_handle_error === "function") {
              fn_handle_error(request);
            }
          }
        }
      };

      request.send();
    }
  };
})();


window.onload = function() {
  IndexManager.bind();
};
